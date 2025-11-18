const defaultControls = {
  "Left": "ArrowLeft",
  "Right": "ArrowRight",
  "Up": "ArrowUp",
  "Down": "ArrowDown",
  "A": "x",
  "B": "z",
  "Select": "Backspace",
  "Start": "Enter"
};
type Controls = typeof defaultControls;
type ControlName = keyof Controls;

const defaultKeybinds = {
  "Zoom in": "+",
  "Zoom out": "-",
  "Fast forward": "Shift",
  "Save state": "s",
  "Load state": "l",
  "Save slot 1": "1",
  "Save slot 2": "2",
  "Save slot 3": "3",
  "Save slot 4": "4",
  "Save slot 5": "5",
  "Save slot 6": "6",
  "Save slot 7": "7",
  "Save slot 8": "8",
  "Save slot 9": "9",
  "Save slot 10": "0",
};
type Keybinds = typeof defaultKeybinds;
type KeybindName = keyof Keybinds;

export default class InputManager {
  public mappingToRebind: string | undefined = $state(undefined);
  private pauseCallback: () => void = () => { };
  private controlCallback: (input: ControlName, pressed: boolean) => void = () => { };
  private keybindCallback: (input: KeybindName, pressed: boolean) => void = () => { };

  public controls: Controls = $state(defaultControls);
  public keybinds: Keybinds = $state(defaultKeybinds);

  constructor() {
    let controls = localStorage.getItem("controls");
    if (controls !== null) {
      this.controls = JSON.parse(controls);
    }
    let keybinds = localStorage.getItem("keybinds");
    if (keybinds !== null) {
      this.keybinds = JSON.parse(keybinds);
    }
  }

  saveMappings = () => {
    localStorage.setItem("controls", JSON.stringify(this.controls));
    localStorage.setItem("keybinds", JSON.stringify(this.keybinds));
  }

  private getKey = (event: KeyboardEvent) => {
    if (event.key === " ") {
      return "Space";
    } else {
      return event.key;
    }
  }

  handleKey = (event: KeyboardEvent, pressed: boolean) => {
    let key = this.getKey(event);
    let controlNames = Object.keys(this.controls) as ControlName[];
    let keybindNames = Object.keys(this.keybinds) as KeybindName[];
    if (this.mappingToRebind) {
      let rebind = this.mappingToRebind;
      this.mappingToRebind = undefined;
      // Escape is reserved for pausing
      if (key === "Escape") {
        return;
      }
      controlNames.forEach(name => {
        if (name === rebind) {
          this.controls[name] = key;
          this.saveMappings();
          return;
        }
      });
      keybindNames.forEach(name => {
        if (name === rebind) {
          this.keybinds[name] = key;
          this.saveMappings();
          return;
        }
      });
    }
    if (pressed && key === "Escape") {
      this.pauseCallback();
      return;
    }
    controlNames.forEach(name => {
      if (this.controls[name] === key) {
        this.controlCallback(name, pressed);
      }
    });
    keybindNames.forEach(name => {
      if (this.keybinds[name] === key) {
        this.keybindCallback(name, pressed);
      }
    });
  }

  onPause(callback: () => void) {
    this.pauseCallback = callback;
  }

  onControlInput(callback: (input: ControlName, pressed: boolean) => void) {
    this.controlCallback = callback;
  }

  onKeybindPressed(callback: (keybind: KeybindName, pressed: boolean) => void) {
    this.keybindCallback = callback;
  }

  setToDefaults = () => {
    this.controls = defaultControls;
    this.keybinds = defaultKeybinds;
    this.saveMappings();
  }
}
