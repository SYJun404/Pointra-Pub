import { useState, useEffect } from "react";
import { platform } from "@tauri-apps/plugin-os";

interface PlatformInfo {
    isMac: boolean;
    isWindows: boolean;
}

export function usePlatform(): PlatformInfo {
    const [platformInfo, setPlatformInfo] = useState<PlatformInfo>({
        isMac: false,
        isWindows: false,
    });

    useEffect(() => {
        const fetchPlatform = () => {
            try {
                const currentPlatform = platform();
                setPlatformInfo({
                    isMac: currentPlatform === "macos",
                    isWindows: currentPlatform === "windows",
                });
            } catch (error) {
                console.error("Failed to get platform info from Tauri:", error);
            }
        };

        fetchPlatform();
    }, []);

    return platformInfo;
}
