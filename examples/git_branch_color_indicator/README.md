# ZSA Keyboard Branch Color Indicator

This script changes your ZSA keyboard's LED colors based on your git branch to provide a visual indicator when you're working in protected branches like master or main.

## Requirements

- [Keymapp](https://www.zsa.io/flash/)
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

### Keymapp Management

The script handles Keymapp (the ZSA keyboard management application) automatically:

- Checks if Keymapp is running using `pgrep`
- Launches Keymapp if needed using macOS's `open` command
- Uses AppleScript to move the Keymapp window off-screen
- Manages keyboard connections using Kontroll CLI commands

### Cross-Platform Considerations

This script uses macOS-specific commands in several places. If you're adapting for another OS, you'll need to modify:

1. Application Management:

   ```bash
   # macOS command to launch Keymapp
   open -g -a "Keymapp"
   ```

2. Window Management:

   ```bash
   # macOS AppleScript for window control
   osascript -e '
     tell application "System Events"
       tell process "keymapp"
         tell window 1
           set position to {2000, -1000}
         end tell
       end tell
     end tell
   '
   ```

3. Process Detection:
   ```bash
   # macOS process check
   pgrep -q "keymapp"
   ```

For other operating systems:

- Linux users might use `xdotool` or `wmctrl` for window management
- Windows users might use PowerShell commands for process and window control
- The Kontroll CLI commands should work the same across platforms

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

- If colors aren't changing, ensure Keymapp is installed and the keyboard is connected
- If colors aren't changing after modifying the script, you may need to open a new terminal or source the script to reresh.
- If you see no output when switching branches, that's normal! The script runs silently unless there's an error
- If you need to manually reconnect: `kontroll connect-any`
