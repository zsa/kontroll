#!/usr/bin/env bash

# Configuration
# Add any branches that should trigger the protected branch color
PRIMARY_BRANCHES=("master" "main")

# Colors should be specified in hex format WITHOUT the '#' symbol
PROTECTED_BRANCH_COLOR="E56717"  # Papaya Orange
FEATURE_BRANCH_COLOR="25A2DC"    # Yas Marina Blue

evaluate_directory_and_branch() {
  if git rev-parse --is-inside-work-tree &> /dev/null; then
      current_branch=$(git branch --show-current)
      # Check if current branch is in PRIMARY_BRANCHES array
      if [[ " ${PRIMARY_BRANCHES[@]} " =~ " ${current_branch} " ]]; then
        alert_master
      else
        call_off_alert
      fi
    fi
}

# Function to detect branch change
detect_branch_change() {
  if git rev-parse --is-inside-work-tree &> /dev/null; then
    new_branch=$(git branch --show-current)
    if [[ "$new_branch" != "$last_branch" ]]; then
      last_branch=$new_branch
      evaluate_directory_and_branch
    fi
  fi
}

ensure_keyboard_connection() {
  # Check if Keymapp is running
  if ! pgrep -q "keymapp"; then
    echo "Keymapp is not running. Starting it..."
    open -g -a "Keymapp"
    # Wait longer for Keymapp to fully start
    sleep 2
    # Move window off screen (effectively hiding it)
    osascript -e '
      tell application "System Events"
        tell process "keymapp"
          tell window 1
            set position to {2000, -1000}
          end tell
        end tell
      end tell
    ' &>/dev/null
  fi

  # Give a few attempts to check for connection
  for i in {1..3}; do
    if kontroll list 2>/dev/null | grep -q "(connected)"; then
      return 0
    elif [ $i -lt 3 ]; then
      kontroll connect-any 2>/dev/null
      sleep 1
    fi
  done

  echo "Could not connect to keyboard. Is it plugged in?"
  return 1
}

alert_master() {
  if ensure_keyboard_connection; then
    kontroll set-rgb-all --color "E56717" &> /dev/null
  fi
}

call_off_alert() {
  if ensure_keyboard_connection; then
    kontroll set-rgb-all --color "#25A2DC" &> /dev/null
  fi
}


chpwd() {
  evaluate_directory_and_branch
}

# Initialize last_branch variable
last_branch=""

monitor_branch_changes() {
  # Add initial delay to avoid double execution
  sleep 1
  while true; do
    detect_branch_change
    sleep 1
  done
}

# Start the branch change monitor in the background using a subshell
(monitor_branch_changes >/dev/null 2>&1 &)

# Also run the function when a new terminal session starts
evaluate_directory_and_branch