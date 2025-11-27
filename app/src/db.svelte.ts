import Dexie, { type EntityTable } from "dexie"

interface StateData {
  id: number,
  romHash: number,
  slot: number,
  state: ArrayBuffer
}

interface SaveData {
  id: number,
  romHash: number,
  ram: ArrayBuffer,
}

type DexieDB = Dexie & {
  states: EntityTable<StateData, "id">,
  saves: EntityTable<SaveData, "id">,
}

interface SerializedDB {
  states: (Omit<StateData, "state"> & { state: string })[]
  saves: (Omit<SaveData, "ram"> & { ram: string })[]
}

export class Database {
  private db: DexieDB;

  constructor() {
    const db = new Dexie("DMG-2025") as DexieDB;
    db.version(1).stores({
      states: "++id, romHash, slot",
      saves: "++id, romHash"
    })
    this.db = db;
  }

  private getStateCollection = (romHash: number, slot: number) => {
    return this.db.states.filter(state => state.romHash === romHash && state.slot === slot);
  }

  saveState = async (romHash: number, slot: number, state: Uint8Array) => {
    // Delete previous state
    let collection = this.getStateCollection(romHash, slot);
    await collection.delete();
    // Save new state
    await this.db.states.add({
      romHash,
      slot,
      state: state.buffer,
    });
  }

  getState = async (romHash: number, slot: number) => {
    let collection = this.getStateCollection(romHash, slot);
    const result = await collection.first();
    if (result !== undefined) {
      return new Uint8Array(result.state);
    }
    else {
      throw `State not found for slot ${slot}`;
    }
  }

  private getSaveCollection = (romHash: number) => {
    return this.db.saves.filter(state => state.romHash === romHash);
  }

  saveRAM = async (romHash: number, ram: Uint8Array) => {
    // Delete previous save 
    let collection = this.getSaveCollection(romHash);
    await collection.delete();
    // Save new save 
    await this.db.saves.add({
      romHash,
      ram: ram.buffer,
    });
  }

  getRAM = async (romHash: number) => {
    let collection = this.getSaveCollection(romHash);
    const result = await collection.first();
    if (result !== undefined) {
      return new Uint8Array(result.ram);
    }
    else {
      throw `Saved RAM not found`;
    }
  }

  /// Encodes an ArrayBuffer into Base64
  private encodeBuffer = (buffer: ArrayBuffer) => {
    let bytes = new Uint8Array(buffer);
    // Convert bytes into binary string
    let binaryString = "";
    bytes.forEach(byte => {
      binaryString += String.fromCharCode(byte);
    })
    // Encode to Base64
    return btoa(binaryString);
  }

  /// Decodes an ArrayBuffer from Base64
  private decodeBuffer = (string: string) => {
    // Decode Base64
    let binaryString = atob(string);
    // Construct RAM from binary string
    let buffer = new Uint8Array(new ArrayBuffer(binaryString.length));
    for (let i = 0; i < binaryString.length; i++) {
      buffer[i] = binaryString.charCodeAt(i);
    }
    return buffer.buffer;
  }

  /// Serializes database into a JSON string
  serializeData = async () => {
    let serialized: SerializedDB | undefined = undefined;
    await this.db.transaction('r', ["saves", "states"], async () => {
      // Map saved buffers into Base64
      serialized = {
        states: (await this.db.states.toArray()).map(state => {
          return {
            state: this.encodeBuffer(state.state),
            romHash: state.romHash,
            slot: state.slot,
            id: state.id
          };
        }),
        saves: (await this.db.saves.toArray()).map(save => {
          return {
            ram: this.encodeBuffer(save.ram),
            romHash: save.romHash,
            id: save.id
          };
        }),
      };
    });
    if (!serialized) {
      throw "Couldn't complete transaction";
    }
    // Return as JSON
    const jsonString = JSON.stringify(serialized, null, 2);
    console.info("Serialized save data");
    return jsonString;
  }

  /// Overwrites database contents from a serialized JSON string
  deserializeData = async (json: string) => {
    const serialized: SerializedDB = JSON.parse(json);

    await this.db.transaction('rw', ["saves", "states"], async () => {
      // Overwrite states
      let serializedStates = serialized.states.map(state => {
        return {
          // Base64 decode RAM
          state: this.decodeBuffer(state.state),
          romHash: state.romHash,
          slot: state.slot,
          id: state.id,
        }
      })
      await this.db.states.clear();
      await this.db.states.bulkAdd(serializedStates);
      // Overwrite saves
      let serializedSaves = serialized.saves.map(save => {
        return {
          // Base64 decode RAM
          ram: this.decodeBuffer(save.ram),
          romHash: save.romHash,
          id: save.id,
        }
      })
      await this.db.saves.clear();
      await this.db.saves.bulkAdd(serializedSaves);
    });

    console.info("Deserialized save data");
  }

  /// Clears database
  deleteData = async () => {
    await this.db.saves.clear();
    await this.db.states.clear();
  }
}

export default Database;
