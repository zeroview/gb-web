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
    let result = await collection.first();
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
    let result = await collection.first();
    if (result !== undefined) {
      return new Uint8Array(result.ram);
    }
    else {
      throw `Saved RAM not found`;
    }
  }
}

export default Database;
