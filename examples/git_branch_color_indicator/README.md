# ZSA Keyboard Branch Color Indicator

This script changes your ZSA keyboard's LED colors based on your git branch to provide a visual indicator when you're working in protected branches like master or main.

## Requirements

- [Kontroll](https://github.com/zsa/kontroll)

## Installation

1. Clone this repository or download the script

2. Source the script in your shell's rc file (e.g., .zshrc or .bashrc):
   ```bash
   source /path/to/change_keyboard_color_on_master_branch.bash
   ```

## How It Works

### Git Branch Monitoring

The script continuously monitors your git environment through two mechanisms:

- A background process that checks for branch changes
- A directory change hook that evaluates the git status when you navigate directories

### Color Management

When a branch change is detected, the script:

- Checks if the current branch matches any in your `PRIMARY_BRANCHES` array
- Sets the keyboard color using Kontroll's `set-rgb-all` command to the color specified in the `PROTECTED_BRANCH_COLOR` variable, or the `FEATURE_BRANCH_COLOR` variable if the branch is not in the `PRIMARY_BRANCHES` array

### Keyboard Connection

Kontroll 2 talks to the keyboard directly, so Keymapp is not needed:

- Before setting a color, the script runs `kontroll list` to check that a keyboard is found
- If none is found, it prints "Could not connect to keyboard. Is it plugged in?" and skips the color change

### Cross-Platform Considerations

The script uses only bash, git and Kontroll, so it is no longer tied to macOS.

## Configuration

The script can be customized by modifying the following variables at the top of the file:

```bash
# Add any branches that should trigger the protected branch color
PRIMARY_BRANCHES=("master" "main" "production")

# Colors should be specified in hex format WITHOUT the '#' symbol
PROTECTED_BRANCH_COLOR="E56717"  # Papaya Orange
FEATURE_BRANCH_COLOR="25A2DC"    # Yas Marina Blue
```

## Troubleshooting

- If colors aren't changing, run `kontroll list` to check that your keyboard is found. A Wireless Voyager can also be reached over Bluetooth once it is bonded in your system's Bluetooth settings
- If colors aren't changing after modifying the script, you may need to open a new terminal or source the script to reresh.
- If you see no output when switching branches, that's normal! The script runs silently unless there's an error
- There is no need to reconnect manually: each Kontroll command opens the keyboard on its own. If you have several keyboards, add `--keyboard <INDEX>` (the index from `kontroll list`) to the script's `kontroll set-rgb-all` calls
