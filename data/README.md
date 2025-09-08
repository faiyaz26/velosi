# App Category Mapping

This directory contains the configuration files for categorizing applications in Velosi Tracker. These files are designed to be community-maintained, allowing open source contributors to improve and expand the app categorization system.

## Files Structure

### `categories.json`

Defines the available categories with their properties:

- `id`: Unique identifier for the category
- `name`: Display name of the category
- `description`: Brief description of what apps belong in this category
- `color`: Hex color code for visual representation
- `icon`: Icon identifier (using Lucide icons)

### `app-mappings.json`

Maps applications to categories:

- `category`: The category ID from categories.json
- `apps`: Array of app names that belong to this category
  - App names can include multiple variations separated by `|`
  - Example: `"Visual Studio Code|Code|vscode"` matches any of these names

## Contributing

### Adding New Categories

1. Edit `categories.json`
2. Add a new category object with all required fields
3. Choose a unique `id` and appropriate `color`
4. Update `app-mappings.json` to include apps for the new category

### Adding App Mappings

1. Edit `app-mappings.json`
2. Find the appropriate category
3. Add the app name to the `apps` array
4. Use `|` to separate multiple name variations
5. Consider common variations like:
   - Full name vs short name
   - With/without version numbers
   - Brand variations

### Guidelines

- App names should match exactly how they appear in the system
- Use the most specific category that fits
- Include common variations and abbreviations
- Test your changes to ensure apps are correctly categorized
- Keep descriptions clear and concise

### Example App Name Variations

```json
"Visual Studio Code|Code|vscode"
"Microsoft Word|Word"
"Google Chrome|Chrome"
"Adobe Photoshop|Photoshop"
```

## Usage in Code

The application loads these files at runtime to:

1. Display available categories with proper colors and icons
2. Automatically categorize tracked applications
3. Provide fallback to "unknown" category for unmapped apps

## File Format

Both files use standard JSON format. Ensure proper syntax when editing:

- Use double quotes for strings
- No trailing commas
- Proper JSON structure validation
