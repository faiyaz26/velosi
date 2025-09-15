use std::collections::HashSet;
use std::net::IpAddr;
use std::process::Command;
use std::sync::Mutex;

#[derive(Clone)]
pub struct FirewallWebsiteBlocker {
    blocked_domains: std::sync::Arc<Mutex<HashSet<String>>>,
    blocking_active: std::sync::Arc<Mutex<bool>>,
}

impl FirewallWebsiteBlocker {
    pub fn new() -> Self {
        Self {
            blocked_domains: std::sync::Arc::new(Mutex::new(HashSet::new())),
            blocking_active: std::sync::Arc::new(Mutex::new(false)),
        }
    }

    pub async fn start_website_blocking(&self, urls: Vec<String>) -> Result<(), String> {
        // Extract domains from URLs
        let domains: Vec<String> = urls.iter().map(|url| self.extract_domain(url)).collect();

        // Store blocked domains
        {
            let mut blocked = self.blocked_domains.lock().unwrap();
            blocked.clear();
            for domain in &domains {
                blocked.insert(domain.clone());
            }
        }

        // Apply firewall rules based on platform
        self.apply_firewall_rules(&domains).await?;

        {
            let mut active = self.blocking_active.lock().unwrap();
            *active = true;
        }

        println!(
            "Firewall-based website blocking started for {} domains",
            domains.len()
        );
        Ok(())
    }

    pub async fn stop_website_blocking(&self) -> Result<(), String> {
        let domains: Vec<String> = {
            let blocked = self.blocked_domains.lock().unwrap();
            blocked.iter().cloned().collect()
        };

        // Remove firewall rules
        self.remove_firewall_rules(&domains).await?;

        // Clear blocked domains
        {
            let mut blocked = self.blocked_domains.lock().unwrap();
            blocked.clear();
        }

        {
            let mut active = self.blocking_active.lock().unwrap();
            *active = false;
        }

        println!("Firewall-based website blocking stopped");
        Ok(())
    }

    pub async fn is_blocking(&self) -> bool {
        let active = self.blocking_active.lock().unwrap();
        *active
    }

    fn extract_domain(&self, url: &str) -> String {
        // Remove protocol if present
        let url = url
            .trim_start_matches("http://")
            .trim_start_matches("https://");

        // Remove path if present
        let domain = url.split('/').next().unwrap_or(url);

        // Remove port if present
        let domain = domain.split(':').next().unwrap_or(domain);

        domain.to_string()
    }

    async fn apply_firewall_rules(&self, domains: &[String]) -> Result<(), String> {
        if cfg!(target_os = "macos") {
            self.apply_macos_firewall_rules(domains).await
        } else if cfg!(target_os = "windows") {
            self.apply_windows_firewall_rules(domains).await
        } else if cfg!(target_os = "linux") {
            self.apply_linux_firewall_rules(domains).await
        } else {
            Err("Unsupported platform for firewall blocking".to_string())
        }
    }

    async fn remove_firewall_rules(&self, domains: &[String]) -> Result<(), String> {
        if cfg!(target_os = "macos") {
            self.remove_macos_firewall_rules(domains).await
        } else if cfg!(target_os = "windows") {
            self.remove_windows_firewall_rules(domains).await
        } else if cfg!(target_os = "linux") {
            self.remove_linux_firewall_rules(domains).await
        } else {
            Err("Unsupported platform for firewall blocking".to_string())
        }
    }

    // macOS implementation using pfctl (Packet Filter)
    async fn apply_macos_firewall_rules(&self, domains: &[String]) -> Result<(), String> {
        // First, resolve domains to IP addresses
        let mut ip_addresses = Vec::new();

        for domain in domains {
            match self.resolve_domain_to_ips(domain).await {
                Ok(mut ips) => ip_addresses.append(&mut ips),
                Err(e) => println!("Warning: Failed to resolve {}: {}", domain, e),
            }
        }

        if ip_addresses.is_empty() {
            return Ok(()); // No IPs to block
        }

        // Create pfctl rule file
        let rule_content = self.generate_pfctl_rules(&ip_addresses);

        // Write rules to temporary file
        let rules_file = "/tmp/velosi_block_rules.conf";
        std::fs::write(rules_file, rule_content)
            .map_err(|e| format!("Failed to write pfctl rules: {}", e))?;

        // Apply pfctl rules (this might require user permission the first time)
        let output = Command::new("pfctl")
            .args(["-f", rules_file])
            .output()
            .map_err(|e| format!("Failed to execute pfctl: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("pfctl failed: {}", stderr));
        }

        // Enable pfctl if not already enabled
        let _ = Command::new("pfctl").args(["-e"]).output();

        Ok(())
    }

    async fn remove_macos_firewall_rules(&self, _domains: &[String]) -> Result<(), String> {
        // Flush pfctl rules related to Velosi
        let output = Command::new("pfctl")
            .args(["-F", "all"])
            .output()
            .map_err(|e| format!("Failed to execute pfctl: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Warning: pfctl flush failed: {}", stderr);
        }

        Ok(())
    }

    // Windows implementation using netsh or Windows Filtering Platform
    async fn apply_windows_firewall_rules(&self, domains: &[String]) -> Result<(), String> {
        // Resolve domains to IP addresses
        let mut ip_addresses = Vec::new();

        for domain in domains {
            match self.resolve_domain_to_ips(domain).await {
                Ok(mut ips) => ip_addresses.append(&mut ips),
                Err(e) => println!("Warning: Failed to resolve {}: {}", domain, e),
            }
        }

        // Create Windows Firewall rules using netsh
        for (i, ip) in ip_addresses.iter().enumerate() {
            let rule_name = format!("Velosi_Block_{}", i);

            let output = Command::new("netsh")
                .args([
                    "advfirewall",
                    "firewall",
                    "add",
                    "rule",
                    &format!("name={}", rule_name),
                    "dir=out",
                    "action=block",
                    &format!("remoteip={}", ip),
                ])
                .output()
                .map_err(|e| format!("Failed to execute netsh: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!(
                    "Warning: Failed to create firewall rule for {}: {}",
                    ip, stderr
                );
            }
        }

        Ok(())
    }

    async fn remove_windows_firewall_rules(&self, _domains: &[String]) -> Result<(), String> {
        // Remove all Velosi firewall rules
        let output = Command::new("netsh")
            .args([
                "advfirewall",
                "firewall",
                "delete",
                "rule",
                "name=Velosi_Block_*",
            ])
            .output()
            .map_err(|e| format!("Failed to execute netsh: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Warning: Failed to remove firewall rules: {}", stderr);
        }

        Ok(())
    }

    // Linux implementation using iptables
    async fn apply_linux_firewall_rules(&self, domains: &[String]) -> Result<(), String> {
        // Resolve domains to IP addresses
        let mut ip_addresses = Vec::new();

        for domain in domains {
            match self.resolve_domain_to_ips(domain).await {
                Ok(mut ips) => ip_addresses.append(&mut ips),
                Err(e) => println!("Warning: Failed to resolve {}: {}", domain, e),
            }
        }

        // Create iptables rules
        for ip in ip_addresses {
            let output = Command::new("iptables")
                .args([
                    "-A",
                    "OUTPUT",
                    "-d",
                    &ip.to_string(),
                    "-j",
                    "REJECT",
                    "-m",
                    "comment",
                    "--comment",
                    "Velosi website block",
                ])
                .output()
                .map_err(|e| format!("Failed to execute iptables: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!(
                    "Warning: Failed to create iptables rule for {}: {}",
                    ip, stderr
                );
            }
        }

        Ok(())
    }

    async fn remove_linux_firewall_rules(&self, _domains: &[String]) -> Result<(), String> {
        // Remove all Velosi iptables rules
        let output = Command::new("iptables")
            .args([
                "-D",
                "OUTPUT",
                "-m",
                "comment",
                "--comment",
                "Velosi website block",
                "-j",
                "REJECT",
            ])
            .output()
            .map_err(|e| format!("Failed to execute iptables: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Warning: Failed to remove iptables rules: {}", stderr);
        }

        Ok(())
    }

    async fn resolve_domain_to_ips(&self, domain: &str) -> Result<Vec<IpAddr>, String> {
        use std::net::ToSocketAddrs;

        // Try to resolve domain to IP addresses
        let addresses = format!("{}:80", domain)
            .to_socket_addrs()
            .map_err(|e| format!("Failed to resolve {}: {}", domain, e))?
            .map(|addr| addr.ip())
            .collect::<std::collections::HashSet<_>>() // Remove duplicates
            .into_iter()
            .collect();

        Ok(addresses)
    }

    fn generate_pfctl_rules(&self, ip_addresses: &[IpAddr]) -> String {
        let mut rules = String::new();

        // Add header
        rules.push_str("# Velosi Website Blocker Rules\n");
        rules.push_str("# Block outgoing connections to specified IPs\n\n");

        // Add block rules for each IP
        for ip in ip_addresses {
            rules.push_str(&format!("block out quick to {}\n", ip));
        }

        rules
    }

    pub fn check_firewall_permissions(&self) -> Result<(), String> {
        if cfg!(target_os = "macos") {
            // Check if we can run pfctl
            let output = Command::new("pfctl")
                .args(["-s", "info"])
                .output()
                .map_err(|e| format!("Cannot access pfctl: {}. Please ensure Velosi has the necessary permissions.", e))?;

            if !output.status.success() {
                return Err("Cannot access firewall controls. Please grant Velosi the necessary permissions in System Preferences > Security & Privacy > Privacy > Full Disk Access.".to_string());
            }
        } else if cfg!(target_os = "windows") {
            // Check if we can run netsh (usually requires admin)
            let output = Command::new("netsh")
                .args(["advfirewall", "show", "allprofiles"])
                .output()
                .map_err(|e| format!("Cannot access Windows Firewall: {}", e))?;

            if !output.status.success() {
                return Err("Cannot access Windows Firewall. Please run Velosi as administrator or grant it the necessary permissions.".to_string());
            }
        } else if cfg!(target_os = "linux") {
            // Check if we can run iptables (usually requires sudo)
            let output = Command::new("iptables")
                .args(["-L", "-n"])
                .output()
                .map_err(|e| format!("Cannot access iptables: {}", e))?;

            if !output.status.success() {
                return Err("Cannot access iptables. Please run Velosi with sudo or grant it the necessary permissions.".to_string());
            }
        }

        Ok(())
    }
}
