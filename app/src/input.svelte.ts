export type ControlName = "Left" | "Right" | "Up" | "Down" | "A" | "B" | "Select" | "Start";
export type KeybindName = "Zoom in" | "Zoom out";

const defaultControls = {
  "Left": "ArrowLeft",
  "Right": "ArrowRight",
  "Up": "ArrowUp",
  "Down": "ArrowDown",
  "A": "x",
  "B": "z",
  "Select": "Backspace",
  "Start": "Enter"
}
type Controls = typeof defaultControls;
const defaultKeybinds = {
  "Zoom in": "+",
  "Zoom out": "-"
}
type Keybinds = typeof defaultKeybinds;

export default class InputManager {
  public mappingToRebind: string | undefined = $state(undefined);
  private pauseCallback: () => void = () => { };
  private controlCallback: (input: ControlName, pressed: boolean) => void = () => { };
  private keybindCallback: (input: KeybindName) => void = () => { };

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
    if (pressed) {
      keybindNames.forEach(name => {
        if (this.keybinds[name] === key) {
          this.keybindCallback(name);
        }
      });
    }
  }

  onPause(callback: () => void) {
    this.pauseCallback = callback;
  }

  onControlInput(callback: (input: ControlName, pressed: boolean) => void) {
    this.controlCallback = callback;
  }

  onKeybindPressed(callback: (keybind: KeybindName) => void) {
    this.keybindCallback = callback;
  }

  setToDefaults = () => {
    this.controls = defaultControls;
    this.keybinds = defaultKeybinds;
    this.saveMappings();
  }
}
