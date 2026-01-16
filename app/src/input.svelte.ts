/// Default emulator controls
const defaultControls = {
  "D-Pad Left": "Left",
  "D-Pad Right": "Right",
  "D-Pad Up": "Up",
  "D-Pad Down": "Down",
  "A": "X",
  "B": "Z",
  "Select": "Backspace",
  "Start": "Enter"
};
type Controls = typeof defaultControls;
type ControlName = keyof Controls;

/// Default keybinds
const defaultKeybinds = {
  "Fast forward": "F",
  "Save state": "S",
  "Load state": "L",
  "State slot 1": "1",
  "State slot 2": "2",
  "State slot 3": "3",
  "State slot 4": "4",
  "State slot 5": "5",
  "State slot 6": "6",
  "State slot 7": "7",
  "State slot 8": "8",
  "State slot 9": "9",
  "State slot 10": "0",
};
type Keybinds = typeof defaultKeybinds;
type KeybindName = keyof Keybinds;

export default class InputManager {
  public mappingToRebind: string | undefined = $state(undefined);
  private pauseCallback: () => void = () => { };
  private controlCallback: (input: ControlName, pressed: boolean) => void = () => { };
  private keybindCallback: (input: KeybindName, pressed: boolean) => void = () => { };
  private keyboardFocusEndCallback: () => void = () => { };

  public keyboardFocused = false;
  public controls: Controls = $state(defaultControls);
  public keybinds: Keybinds = $state(defaultKeybinds);

  constructor() {
    // Load input map from LocalStorage
    let controls = localStorage.getItem("controls");
    if (controls !== null) {
      this.controls = JSON.parse(controls);
    }
    let keybinds = localStorage.getItem("keybinds");
    if (keybinds !== null) {
      this.keybinds = JSON.parse(keybinds);
    }
  }

  /// Saves input map to LocalStorage
  saveMappings = () => {
    localStorage.setItem("controls", JSON.stringify(this.controls));
    localStorage.setItem("keybinds", JSON.stringify(this.keybinds));
  }

  /// Returns event from key and modifies it for use
  private getKey = (event: KeyboardEvent) => {
    // Save Space as a string so it can be actually displayed as text
    if (event.key === " ") {
      return "Space";
    }
    /// Return key as upper case to disable checking for casing and also for better displaying 
    if (event.key.length == 1) {
      return event.key.toUpperCase();
    }
    /// Slice off Arrow for better displaying
    if (event.key.slice(0, 5) == "Arrow") {
      return event.key.slice(5);
    }
    return event.key;
  }

  handleKey = (event: KeyboardEvent, pressed: boolean) => {
    let key = this.getKey(event);

    // Disable input when keyboard is focused on something (probably a textbox)
    if (this.keyboardFocused) {
      // Remove keyboard focus when pressing Escape or Enter
      if (key === "Escape" || key === "Enter") {
        this.keyboardFocused = false;
        this.keyboardFocusEndCallback();
      }
      return;
    }

    let controlNames = Object.keys(this.controls) as ControlName[];
    let keybindNames = Object.keys(this.keybinds) as KeybindName[];

    // Handle keybind remapping
    if (this.mappingToRebind) {
      if (!pressed) {
        return;
      }

      let rebind = this.mappingToRebind;
      this.mappingToRebind = undefined;
      // Use Escape to cancel keybind rebinding
      if (key === "Escape") {
        return;
      }
      // Loop through keybind names and replace the key of the one that matches
      controlNames.forEach(name => {
        if (name === rebind) {
          this.controls[name] = key;
          this.saveMappings();
        }
      });
      keybindNames.forEach(name => {
        if (name === rebind) {
          this.keybinds[name] = key;
          this.saveMappings();
        }
      });
      return;
    }
    /// Use Escape for pausing
    if (pressed && key === "Escape") {
      this.pauseCallback();
      return;
    }

    // Disable repeating events for emulator input
    if (!event.repeat) {
      controlNames.forEach(name => {
        if (this.controls[name] === key) {
          this.controlCallback(name, pressed);
        }
      });
    }
    keybindNames.forEach(name => {
      if (this.keybinds[name] === key) {
        this.keybindCallback(name, pressed);
      }
    });
  }

  /// Called when pause button (Escape) is pressed
  onPause(callback: () => void) {
    this.pauseCallback = callback;
  }

  /// Called when a control input (for emulator) is pressed or released
  onControlInput(callback: (input: ControlName, pressed: boolean) => void) {
    this.controlCallback = callback;
  }

  /// Called when a keybind is pressed or released
  onKeybindPressed(callback: (keybind: KeybindName, pressed: boolean) => void) {
    this.keybindCallback = callback;
  }

  /// Called when keyboard focus is removed with Escape or Enter
  onKeyboardFocusEnd(callback: () => void) {
    this.keyboardFocusEndCallback = callback;
  }

  /// Set saved keybinds to their defaults
  setToDefaults = () => {
    this.controls = defaultControls;
    this.keybinds = defaultKeybinds;
    this.saveMappings();
  }
}
