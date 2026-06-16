import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

/**
 * 自定义 Hook：将最新值同步到 Ref 中，避免 useEffect 依赖频繁触发
 */
function useLatest<T>(value: T) {
    const ref = useRef(value);
    ref.current = value;
    return ref;
}

export function useWindowListener(
    isPinned: boolean,
    setIsPinned: (isPinned: boolean) => void,
    hideKey: string,
    pinKey: string,
) {
    // 将所有频繁改变的属性收纳到一个 ref 对象中
    const stateRef = useLatest({ isPinned, hideKey, pinKey });

    useEffect(() => {
        const appWindow = getCurrentWindow();

        const handleMouseEnter = () => {
            invoke("update_hover_status", { hovered: true });
        };

        const handleMouseLeave = () => {
            if (!stateRef.current.isPinned) {
                invoke("update_hover_status", { hovered: false });
                appWindow.hide();
            }
        };

        const handleKeydown = (e: KeyboardEvent) => {
            const { hideKey, pinKey, isPinned } = stateRef.current;

            if (e.key === hideKey) {
                appWindow.hide();
            }
            if (e.key === pinKey) {
                setIsPinned(!isPinned);
            }
        };

        const isMac =
            navigator.platform?.toLowerCase().includes("mac") ?? false;
        const docElment = isMac ? document.body : document;

        // 绑定事件
        docElment.addEventListener("mouseenter", handleMouseEnter);
        docElment.addEventListener("mouseleave", handleMouseLeave);
        document.addEventListener("keydown", handleKeydown);

        // 清理事件
        return () => {
            docElment.removeEventListener("mouseenter", handleMouseEnter);
            docElment.removeEventListener("mouseleave", handleMouseLeave);
            document.removeEventListener("keydown", handleKeydown);
        };
    }, []);
}

export function useOnWindowChange(onHide: () => void) {
    const savedOnHide = useRef(onHide);

    useEffect(() => {
        savedOnHide.current = onHide;
    }, [onHide]);

    useEffect(() => {
        async function setupListener() {
            const appWindow = getCurrentWindow();

            // 监听窗口隐藏/失焦事件
            const unlisten = await appWindow.listen("tauri://blur", () => {
                // 判断窗口是否为隐藏状态
                appWindow.isVisible().then((isShow) => {
                    if (!isShow) {
                        savedOnHide.current();
                    }
                });
            });
            return unlisten;
        }

        const listenerPromise = setupListener();
        return () => {
            listenerPromise.then((unlisten) => unlisten());
        };
    }, []);
}
