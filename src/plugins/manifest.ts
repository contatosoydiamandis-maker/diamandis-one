export const PLUGIN_API_VERSION = 1;

export const PLUGIN_PERMISSIONS = [
  "ui.panel",
  "tool.action",
  "filesystem.read",
  "filesystem.write",
  "process.launch"
] as const;

export type PluginPermission = typeof PLUGIN_PERMISSIONS[number];

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  apiVersion: number;
  description: string;
  permissions: PluginPermission[];
}

export type ManifestValidation =
  | { valid: true; manifest: PluginManifest }
  | { valid: false; error: string };

const permissionSet = new Set<string>(PLUGIN_PERMISSIONS);
const pluginIdPattern = /^[a-z0-9][a-z0-9.-]{1,63}$/;
const versionPattern = /^(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)(?:-[0-9A-Za-z.-]+)?$/;

export function validatePluginManifest(input: unknown): ManifestValidation {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    return { valid: false, error: "O manifesto precisa ser um objeto JSON." };
  }

  const value = input as Record<string, unknown>;
  if (typeof value.id !== "string" || !pluginIdPattern.test(value.id)) {
    return { valid: false, error: "ID inválido. Use 2 a 64 caracteres minúsculos, números, ponto ou hífen." };
  }
  if (typeof value.name !== "string" || !value.name.trim() || value.name.length > 80) {
    return { valid: false, error: "O nome precisa ter entre 1 e 80 caracteres." };
  }
  if (typeof value.version !== "string" || !versionPattern.test(value.version)) {
    return { valid: false, error: "Versão inválida. Use o formato semântico MAJOR.MINOR.PATCH." };
  }
  if (value.apiVersion !== PLUGIN_API_VERSION) {
    return { valid: false, error: `Versão de API incompatível. Esta versão aceita API ${PLUGIN_API_VERSION}.` };
  }
  if (typeof value.description !== "string" || value.description.length > 240) {
    return { valid: false, error: "A descrição precisa ser texto com no máximo 240 caracteres." };
  }
  if (!Array.isArray(value.permissions) || value.permissions.length > PLUGIN_PERMISSIONS.length) {
    return { valid: false, error: "A lista de permissões é inválida." };
  }

  const permissions: PluginPermission[] = [];
  for (const permission of value.permissions) {
    if (typeof permission !== "string" || !permissionSet.has(permission)) {
      return { valid: false, error: `Permissão não reconhecida: ${String(permission)}.` };
    }
    if (permissions.includes(permission as PluginPermission)) {
      return { valid: false, error: `Permissão duplicada: ${permission}.` };
    }
    permissions.push(permission as PluginPermission);
  }

  return {
    valid: true,
    manifest: {
      id: value.id,
      name: value.name.trim(),
      version: value.version,
      apiVersion: PLUGIN_API_VERSION,
      description: value.description,
      permissions
    }
  };
}

export class PluginRegistry {
  private readonly manifests = new Map<string, PluginManifest>();

  register(input: unknown): ManifestValidation {
    const result = validatePluginManifest(input);
    if (!result.valid) return result;
    if (this.manifests.has(result.manifest.id)) {
      return { valid: false, error: `Já existe um plugin registrado com o ID "${result.manifest.id}".` };
    }
    this.manifests.set(result.manifest.id, result.manifest);
    return result;
  }

  list(): PluginManifest[] {
    return [...this.manifests.values()].sort((a, b) => a.name.localeCompare(b.name, "pt-BR"));
  }

  get(id: string): PluginManifest | undefined {
    return this.manifests.get(id);
  }

  unregister(id: string): boolean {
    return this.manifests.delete(id);
  }
}

export const pluginRegistry = new PluginRegistry();
