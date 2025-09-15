<div align="center">
  <img src="./public/Velosi.png" alt="Velosi Tracker Logo" width="120" height="120">

# Velosi Tracker

_Boost your productivity with intelligent app and website blocking_

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/faiyaz26/velosi/complete-test-suite.yml)
![TypeScript](https://img.shields.io/badge/TypeScript-007ACC?logo=typescript&logoColor=white)
![React](https://img.shields.io/badge/React-20232A?logo=react&logoColor=61DAFB)
![Tauri](https://img.shields.io/badge/Tauri-24C8DB?logo=tauri&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)

[📥 Download](#installation) • [📖 Documentation](#features) • [🤝 Contributing](#contributing)

</div>

---

## ✨ What is Velosi Tracker?

**Velosi Tracker** is a powerful productivity application that helps you stay focused and track your digital habits. Whether you're working on important tasks or trying to break bad browsing habits, Velosi gives you the tools to take control of your time.

### 🎯 Key Features

- **🛡️ Smart Blocking**: Block distracting websites and applications during focus sessions
- **📊 Activity Tracking**: Monitor your app usage and browsing patterns
- **🎨 Beautiful Dashboard**: Visualize your productivity data with interactive charts
- **🌙 Dark/Light Mode**: Comfortable viewing in any environment
- **🔒 Privacy First**: Your data stays completely local - nothing is sent anywhere unless you choose to export it
- **⚡ Fast & Lightweight**: Built with modern technologies for optimal performance

## 🚀 Features

### Website & App Blocking

- **Focus Mode**: Temporarily block distracting websites and apps
- **Proxy-Based Blocking**: Advanced website blocking using local proxy

### Productivity Analytics

- **Activity Dashboard**: Real-time insights into your digital habits
- **Time Tracking**: Detailed reports on app and website usage
- **Category Analysis**: Group activities by productivity categories
- **Heatmaps**: Visual representation of your daily activity patterns

### Privacy & Security

- **100% Local**: All data stored locally on your device
- **No Telemetry**: No tracking or data collection without your consent

## 🛠️ Tech Stack

### Frontend

- **React 18** - Modern UI framework with hooks
- **TypeScript** - Type-safe JavaScript
- **Tailwind CSS** - Utility-first CSS framework
- **Vite** - Fast build tool and dev server
- **Lucide Icons** - Beautiful icon library

### Backend

- **Tauri 2** - Cross-platform desktop app framework
- **Rust** - High-performance systems programming
- **SQLite** - Local database for data persistence
- **Tokio** - Async runtime for concurrent operations

### Development Tools

- **pnpm** - Fast package manager
- **ESLint** - Code linting
- **Prettier** - Code formatting
- **GitHub Actions** - CI/CD pipeline

## 📦 Installation

### Download

Choose the appropriate version for your operating system:

- **macOS**: Download `.dmg` file
- **Windows**: Download `.msi` or `.exe` installer
- **Linux**: Download `.deb` or `.AppImage` file

### From Source

```bash
# Clone the repository
git clone https://github.com/faiyaz26/velosi.git
cd velosi

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

## 🎮 Usage

### Getting Started

1. **Launch Velosi Tracker**
2. **Grant Accessibility Permissions** (required for app tracking)
3. **Configure Focus Mode** settings
4. **Start Tracking** your productivity

### Focus Mode Setup

1. Go to **Settings** → **Focus Mode**
2. **Add websites** to block during focus sessions
3. **Select apps** to restrict
4. **Set duration** and **start** your focus session

### Viewing Analytics

- **Dashboard**: Overview of your productivity metrics
- **Activity Log**: Detailed timeline of your activities
- **Reports**: Generate custom productivity reports

## 🔧 Configuration

### Proxy Settings

Configure the local proxy server for advanced website blocking:

- **Port**: Default 62828 (configurable)
- **Auto-start**: Enable/disable proxy on app launch
- **Block Lists**: Manage blocked domains

### Categories

Organize your activities with custom categories:

- **Work**: Professional tasks and applications
- **Entertainment**: Leisure and recreational activities
- **Social**: Communication and social media
- **Learning**: Educational content and tools

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Fork and clone
git clone https://github.com/your-username/velosi.git
cd velosi

# Install dependencies
pnpm install

# Start development
pnpm tauri dev
```

### Testing

```bash
# Run all tests
pnpm test

# Run specific test suites
pnpm test:unit      # Unit tests
pnpm test:integration # Integration tests
pnpm test:e2e       # End-to-end tests
```

## 📄 License

This project is licensed under the **GPL v3 License** - see the [LICENSE](LICENSE) file for details.

```
Copyright (C) 2025 Ahmad Faiyaz

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
```

## 🙏 Acknowledgments

- **Tauri Team** - For the amazing cross-platform framework
- **React Community** - For the powerful UI library
- **Rust Community** - For the excellent systems programming language
- **Open Source Contributors** - For making this project possible

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/faiyaz26/velosi/issues)
- **Discussions**: [GitHub Discussions](https://github.com/faiyaz26/velosi/discussions)
- **Documentation**: [Wiki](https://github.com/faiyaz26/velosi/wiki)

---

<div align="center">

**Made with ❤️ using Tauri, React & Rust**

[⭐ Star us on GitHub](https://github.com/faiyaz26/velosi) • [🐛 Report Issues](https://github.com/faiyaz26/velosi/issues) • [💬 Join Discussions](https://github.com/faiyaz26/velosi/discussions)

</div>
