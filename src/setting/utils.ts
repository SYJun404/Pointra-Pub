const MODIFIERS = ["CONTROL", "ALT", "SHIFT", "SUPER"];

export const isSingleKeyShortcut = (id: string) =>
    id === "point_key" || id === "pinned_key" || id === "hide_win_key";

export function eventToKeys(event: KeyboardEvent): string[] {
    const keys = [];
    if (event.ctrlKey) keys.push("CONTROL");
    if (event.altKey) keys.push("ALT");
    if (event.shiftKey) keys.push("SHIFT");
    if (event.metaKey) keys.push("SUPER");

    if (
        !event.code ||
        event.code === "Unidentified" ||
        /^(Control|Alt|Shift|Meta)(Left|Right)$/.test(event.code)
    ) {
        return [];
    }
    return [...keys, event.code];
}

export function normalizeKeys(keys: string[]): string[] {
    if (keys.length === 0) return [];
    const primary = keys[keys.length - 1];
    const modifiers = keys.slice(0, -1);
    return [
        ...MODIFIERS.filter((key) => modifiers.includes(key)),
        ...modifiers.filter((key) => !MODIFIERS.includes(key)).sort(),
        primary === "Esc" ? "Escape" : primary,
    ];
}

export function shortcutSignature(keys: string[]): string {
    return normalizeKeys(keys).join("+");
}

export function getConflictIds(
    shortcuts: { id: string; keys: string[] }[],
): Set<string> {
    const map = new Map<string, string[]>();
    for (const shortcut of shortcuts) {
        const signature = shortcutSignature(shortcut.keys);
        if (!signature) continue;

        const ids = map.get(signature) ?? [];
        ids.push(shortcut.id);
        map.set(signature, ids);
    }
    const conflictIds = new Set<string>();
    for (const ids of map.values()) {
        if (ids.length > 1) ids.forEach((id) => conflictIds.add(id));
    }
    return conflictIds;
}
