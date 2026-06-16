import { MODIFIER_MAP_RUST } from "./settingsMeta";

/** 将 KeyboardEvent 转成规范 keys 数组 */
const codeToKey = (code: string, fallbackKey: string): string => {
    // 1. 处理字母键 (KeyA - KeyZ -> A - Z)
    if (code.startsWith("Key")) {
        return code.slice(3).toUpperCase();
    }

    // 2. 处理数字键 (Digit0 - Digit9 -> 0 - 9)
    if (code.startsWith("Digit")) {
        return code.slice(5);
    }

    // 3. 处理小键盘数字 (Numpad0 - Numpad9)
    if (code.startsWith("Numpad") && code.length === 7) {
        return code.slice(6);
    }

    // 4. 处理常见特殊按键的映射
    const specialKeysMap: Record<string, string> = {
        Space: "Space",
        Enter: "Enter",
        Tab: "Tab",
        Escape: "Esc",
        Backspace: "Backspace",
        Delete: "Delete",
        ArrowUp: "Up",
        ArrowDown: "Down",
        ArrowLeft: "Left",
        ArrowRight: "Right",
        Minus: "-",
        Equal: "=",
        BracketLeft: "[",
        BracketRight: "]",
        Semicolon: ";",
        Quote: "'",
        Backquote: "`",
        Comma: ",",
        Period: ".",
        Slash: "/",
        Backslash: "\\",
    };

    return specialKeysMap[code] || fallbackKey;
};

/** 将 KeyboardEvent 转成规范 keys 数组 */
export const eventToKeys = (
    e: React.KeyboardEvent,
    isMac: boolean,
): string[] => {
    const keys: string[] = [];

    // 1. 检测并添加修饰键 (依顺序添加，保证一致性)
    if (e.ctrlKey) keys.push("Ctrl");
    if (e.altKey) keys.push(isMac ? "Opt" : "Alt");
    if (e.shiftKey) keys.push("Shift");
    if (e.metaKey) keys.push(isMac ? "Cmd" : "Win");

    // 2. 识别主按键
    const isModifierOnly =
        e.key === "Control" ||
        e.key === "Shift" ||
        e.key === "Alt" ||
        e.key === "Meta";

    if (!isModifierOnly) {
        // 使用 e.code 转换，若转换不出则降级使用 e.key
        const primaryKey = codeToKey(e.code, e.key);

        // 避免因判定不一致导致将修饰键再次加入
        const isAlreadyAdded = keys.includes(primaryKey);
        if (primaryKey && !isAlreadyAdded) {
            keys.push(primaryKey);
        }
    }

    return keys;
};

/** 检测快捷键冲突，返回存在冲突的 id 集合 */
export function getConflictIds(
    shortcuts: { id: string; keys: string[] }[],
): Set<string> {
    const map = new Map<string, string[]>();
    for (const s of shortcuts) {
        if (s.keys.length === 0) continue;

        let key = "";
        for (const item of s.keys) {
            if (key === "") {
                key = MODIFIER_MAP_RUST[item] ?? item; // 第一个元素直接赋值
            } else {
                key += "+" + (MODIFIER_MAP_RUST[item] ?? item); // 后续元素加 "+" 拼接
            }
        }
        if (!map.has(key)) map.set(key, []);
        map.get(key)!.push(s.id);
    }
    const conflictSet = new Set<string>();
    for (const ids of map.values()) {
        if (ids.length > 1) ids.forEach((id) => conflictSet.add(id));
    }
    return conflictSet;
}
/**防止 "Ctrl + A" vs "A + Ctrl" 这种问题 */
export function normalizeKeys(keys: string[]) {
    const modifiers = ["Ctrl", "Cmd", "Alt", "Opt", "Shift"];

    const mods = keys.filter((k) => modifiers.includes(k));
    const others = keys.filter((k) => !modifiers.includes(k));

    return [...mods.sort(), ...others.sort()];
}
