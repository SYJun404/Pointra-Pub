import { Display, UniversalAccess, ArrowRotateLeft } from "@gravity-ui/icons";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";

const PERMISSION_ITEMS = [
    {
        key: "screen_recording",
        label: "屏幕录制",
        description: "使用截屏捕获显示内容，用于光标取词功能",
        icon: Display,
        iconBg: "bg-purple-50",
        iconBorder: "border-purple-200",
        iconColor: "#8b5cf6",
        tip: "点击右侧按钮，可跳转设置页面",
    },
    {
        key: "accessibility",
        label: "辅助功能",
        description: "监听键盘输入，实现全局快捷键及取词操作",
        icon: UniversalAccess,
        iconBg: "bg-cyan-50",
        iconBorder: "border-cyan-200",
        iconColor: "#06b6d4",
        tip: "点击右侧按钮，可跳转设置页面",
    },
] as const;

function PermissionMacSection() {
    const [permissions, setPermissions] = useState([
        { key: "accessibility", granted: false },
        { key: "screen_recording", granted: false },
    ]);

    const restart = async () => {
        try {
            // 触发软件重启
            await relaunch();
        } catch (error) {
            console.error("重启失败:", error);
        }
    };

    const openSysStetting = (kind: string, granted: boolean) => {
        if (granted) return;
        invoke("request_permission", {
            kind: kind === "accessibility" ? 1 : 2,
        });
    };

    useEffect(() => {
        invoke<boolean[]>("check_permission").then((result) => {
            const res = [
                { key: "accessibility", granted: result[0] },
                { key: "screen_recording", granted: result[1] },
            ];
            setPermissions(res);
        });
    }, []);

    return (
        <section>
            <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                    <div className="w-1 h-4 rounded-full bg-mainBlueW" />
                    <h2 className="text-sm font-medium text-mainTitleW">
                        授权许可
                    </h2>
                </div>
                <p className="text-[11px] text-tagSecondW mt-px rounded-md ">
                    Permission
                </p>
            </div>
            <div className="rounded-xl border border-borderMainW divide-y divide-borderSubW bg-white overflow-hidden">
                {PERMISSION_ITEMS.map((item) => {
                    const Icon = item.icon;
                    const perm = permissions.find((p) => p.key === item.key)!;

                    return (
                        <div
                            key={item.key}
                            className="flex items-center justify-between p-3 hover:bg-subBgW transition-colors"
                        >
                            <div className="flex items-center gap-3 min-w-0">
                                <div
                                    className={`flex items-center justify-center w-8 h-8 rounded-lg border ${item.iconBg} ${item.iconBorder} shrink-0`}
                                >
                                    <Icon
                                        width={15}
                                        height={15}
                                        color={item.iconColor}
                                    />
                                </div>
                                <div className="min-w-0">
                                    <div className="flex items-center gap-2">
                                        <p className="text-sm text-mainTitleW">
                                            {item.label}
                                        </p>
                                        {!perm.granted && (
                                            <span className="text-[10px] text-tagSecondW leading-none">
                                                {item.tip}
                                            </span>
                                        )}
                                    </div>
                                    <p className="text-[11px] text-tagSecondW mt-0.5 truncate">
                                        {item.description}
                                    </p>
                                </div>
                            </div>
                            {/* 状态标签 */}
                            <div
                                onClick={() =>
                                    openSysStetting(item.key, perm.granted)
                                }
                                className={`flex items-center justify-center shrink-0 text-[11px] font-medium w-10 h-5 pt-0.5 rounded-full border ${
                                    perm.granted
                                        ? "bg-green-50 border-green-200 text-green-600"
                                        : "bg-red-50 border-red-200 text-red-600 transition-all duration-200 active:scale-90 cursor-pointer"
                                }`}
                            >
                                {perm.granted ? "同意" : "拒绝"}
                            </div>
                        </div>
                    );
                })}
            </div>
            {/* 刷新按钮 两个的granted 都是ture 则不显示 */}
            {!permissions.every((p) => p.granted) && (
                <button
                    onClick={restart}
                    className="mt-3 w-full h-8 flex items-center justify-center gap-1.5 rounded-lg border border-borderMainW
                                   bg-white text-xs text-tagSecondW hover:bg-subBgW hover:text-mainTitleW
                                   transition-all active:scale-[0.98] cursor-pointer"
                >
                    <ArrowRotateLeft width={13} height={13} />
                    重启软件
                </button>
            )}
        </section>
    );
}

export default PermissionMacSection;
