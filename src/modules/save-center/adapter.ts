/**
 * Boundary for integrating the existing Save Manager.
 * Keep its storage format and backup/restore implementation authoritative.
 * The adapter must be implemented against the original project after its source
 * repository/archive is located. Do not create a parallel backup engine here.
 */
export type SaveManagerGame = {
  id: string;
  title: string;
  saveLocations: string[];
};

export type SaveBackup = {
  id: string;
  gameId: string;
  createdAt: string;
  location: string;
  sizeBytes: number;
};

export interface SaveManagerAdapter {
  listSupportedGames(): Promise<SaveManagerGame[]>;
  listBackups(gameId?: string): Promise<SaveBackup[]>;
  createBackup(gameId: string): Promise<SaveBackup>;
  restoreBackup(backupId: string): Promise<void>;
}
