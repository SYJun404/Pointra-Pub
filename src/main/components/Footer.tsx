import {
    Pin,
    Magnifier,
    ChevronDown,
    CircleArrowUpFill,
} from "@gravity-ui/icons";
import logo from "../../assets/icon/pointraInApp.png";
import useUiStore from "../../main/store/useUiStore";
import { useNavigate } from "react-router-dom";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";

function Footer({ path }: { path: string }) {
    const navigate = useNavigate();
    const isPinned = useUiStore((state) => state.isPinned);
    const updateUrl = useUiStore((state) => state.updateUrl);
    const setIsPinned = useUiStore((state) => state.setIsPinned);
    const appWindow = getCurrentWindow();

    const ACTION_BUTTONS = [
        {
            id: "search",
            func: () => appWindow.hide(),
            icon: <ChevronDown color="#bbbbbb" height={14} width={14} />,
        },
        {
            id: "home",
            func: () => navigate("/search"),
            icon: <Magnifier color="#bbbbbb" height={14} width={14} />,
        },
    ];

    const IconButton = ({
        children,
        onClick,
    }: {
        children: React.ReactNode;
        onClick: () => void;
    }) => (
        <div
            onClick={onClick}
            className="flex items-center justify-center w-6 h-6 rounded-md bg-white border border-borderMainW
                       cursor-pointer transition-transform active:scale-90"
        >
            {children}
        </div>
    );

    const openUpdateUrl = async () => {
        await appWindow.hide();
        openUrl(updateUrl);
    };

    return (
        <div className="mt-auto bg-subBgW h-10 px-3 py-2 border-t border-borderSubW">
            <div className="flex items-center h-full relative gap-2">
                <img className="w-4 h-4" src={logo}></img>
                <p className="text-sm text-tagW absolute left-6">Pointra</p>

                <div className="ml-auto flex gap-1.5">
                    {(() => {
                        const activeBtn = ACTION_BUTTONS.find(
                            ({ id }) => id === path,
                        );
                        if (!activeBtn) return null;

                        const { id, icon, func } = activeBtn;
                        return (
                            <IconButton key={id} onClick={func}>
                                {icon}
                            </IconButton>
                        );
                    })()}
                    <div
                        onClick={() => setIsPinned(!isPinned)}
                        className={`
                            flex items-center justify-center w-6 h-6 rounded-md border cursor-pointer
                            transition-all duration-200 active:scale-90
                            ${
                                isPinned
                                    ? "bg-red-50 border-red-200" // 选中状态
                                    : "bg-white  border-borderMainW" // 默认状态
                            }
                          `}
                    >
                        <div
                            className={`transition-transform duration-300 ${!isPinned ? "rotate-0" : "-rotate-45"}`}
                        >
                            <Pin
                                // 根据状态切换颜色
                                color={isPinned ? "#fa2c37" : "#bbbbbb"}
                                width={14}
                                height={14}
                            />
                        </div>
                    </div>

                    {updateUrl !== "" && (
                        <div
                            onClick={openUpdateUrl}
                            className="
                                    flex items-center h-6  cursor-pointer rounded-md border border-emerald-200
                                    transition-all bg-emerald-50 duration-200 gap-1 active:scale-90 px-1"
                        >
                            <CircleArrowUpFill
                                // 根据状态切换颜色
                                color={" oklch(69.6% 0.17 162.48)"}
                                width={16}
                                height={16}
                            />
                            <p className="text-xs text-emerald-500">版本更新</p>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}

export default Footer;
