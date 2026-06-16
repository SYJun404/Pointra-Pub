import { Gear, CircleXmarkFill } from "@gravity-ui/icons";

export default function SettingHeader({ closeApp }: { closeApp: () => void }) {
    return (
        <>
            <div data-tauri-drag-region className="h-3 w-full absolute" />
            <div className="p-3  flex items-center justify-between border-b border-borderSubW shrink-0">
                <div className="flex items-center gap-2">
                    <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-blueBgW border border-blueBorderW">
                        <Gear width={14} height={14} color="#4a90d9" />
                    </div>
                    <h1 className="text-lg font-medium text-mainTitleW">
                        Settings
                    </h1>
                </div>

                <div
                    onClick={closeApp}
                    className="flex items-center justify-center  cursor-pointer"
                >
                    <CircleXmarkFill width={20} height={20} color="#bbbbbb" />
                </div>
            </div>
        </>
    );
}
