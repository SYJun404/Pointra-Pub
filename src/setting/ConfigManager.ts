// ConfigManager.ts
import { invoke } from "@tauri-apps/api/core";
import { platform } from "@tauri-apps/plugin-os";
import { RustAppConfig, GeneralSetting, ShortcutItem } from "./types";
import {
    GENERAL_SETTINGS_TEMPLATE,
    SHORTCUT_LIST_TEMPLATE,
    MODIFIER_MAP_RUST,
} from "./settingsMeta.ts";

export class ConfigManager {
    /**
     * 从后端获取原始的 RustAppConfig
     */
    static async fetchRawConfig(): Promise<RustAppConfig> {
        return await invoke<RustAppConfig>("get_config");
    }

    static async stopShortcuts(): Promise<void> {
        await invoke<void>("stop_shortcuts");
    }

    /**
     * 获取前端渲染所需的完整打包数据
     */
    static async getAllSettings() {
        const rustConfig = await this.fetchRawConfig();

        const general = GENERAL_SETTINGS_TEMPLATE.map((meta) => ({
            ...meta,
            value: rustConfig[meta.id],
        }));

        const shortcuts: ShortcutItem[] = SHORTCUT_LIST_TEMPLATE.filter(
            (meta) => !meta.macOnly || platform() === "macos",
        ).map(
            (meta) => ({
                ...meta,
                keys: meta.rustKeys.flatMap((key) =>
                    key.endsWith("_modifiers")
                        ? rustConfig[key].split("|").map((part) => part.trim()).filter(Boolean)
                        : [rustConfig[key]],
                ),
            }),
        );

        const first_setup_completed = rustConfig.first_setup_completed;

        return { general, shortcuts, first_setup_completed };
    }

    /**
     * 更新所有设置项并同步到后端
     */
    static async updateAllSetting(
        shortcuts: ShortcutItem[],
        general: GeneralSetting[],
    ): Promise<boolean> {
        try {
            // 将 shortcuts 和 general 转换为 RustAppConfig 格式
            // 保留当前平台未展示的设置，以及首次设置完成状态。
            const config = await this.fetchRawConfig();

            for (const setting of general) {
                Object.assign(config, { [setting.id]: setting.value });
            }
            for (const setting of shortcuts) {
                for (const key of setting.save) {
                    // 修饰键允许 Cmd + Shift 等组合，主按键始终取最后一个。
                    const keys = setting.keys.map((value) => MODIFIER_MAP_RUST[value] ?? value);
                    const value = key.id.endsWith("_modifiers")
                        ? keys.slice(0, -1).join(" | ")
                        : keys[keys.length - 1];
                    Object.assign(config, {
                        [key.id]: value,
                    });
                }
            }

            return await invoke<boolean>("update_config", {
                newConfig: config, //小了A阿斯顿笑了
            });
        } catch (error) {
            console.error(error);
            return false;
        }
    }
}
