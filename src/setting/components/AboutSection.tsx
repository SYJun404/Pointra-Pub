import pointraLogo from "../../assets/icon/pointra.png";
import { openUrl } from "@tauri-apps/plugin-opener";

export default function AboutSection() {
    const openTutorial = () => {
        openUrl("https://pointra.syjun.vip");
    };

    return (
        <section>
            <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                    <div className="w-1 h-4 rounded-full bg-mainBlueW" />
                    <h2 className="text-sm font-medium text-mainTitleW">
                        关于软件
                    </h2>
                </div>
                <p className="text-[11px] text-tagSecondW mt-px rounded-md ">
                    About
                </p>
            </div>
            <div className="rounded-xl border border-borderMainW bg-white overflow-hidden p-3">
                <div className="flex items-center gap-3">
                    <div className="flex items-center justify-center w-9 h-9 rounded-lg bg-mainBlueW">
                        <img
                            src={pointraLogo}
                            alt="Logo"
                            className="w-7.5 h-7.5"
                        />
                    </div>
                    <div>
                        <p className="text-sm text-mainTitleW font-medium">
                            Pointra
                        </p>
                        <p className="text-[11px] text-tagSecondW mt-0.5">
                            版本 {__APP_VERSION__} 『 光标所指 · 翻译即达 』
                        </p>
                    </div>

                    <div onClick={openTutorial} className="ml-auto">
                        <div
                            className="text-sm text-mainBlueW px-2 py-1 text-center cursor-pointer transition-transform active:scale-90
                             rounded-md bg-blueBgW border border-blueBorderW"
                        >
                            使用教程
                        </div>
                    </div>
                </div>
            </div>
            <p className="text-[11px] mt-3 text-center text-tagSecondW">
                Developed By SYJun
            </p>
        </section>
    );
}
