export type ModuleId = "overview" | "files" | "toolbox" | "games" | "saves" | "settings";

export interface ModuleItem {
  id: ModuleId;
  label: string;
  description: string;
}