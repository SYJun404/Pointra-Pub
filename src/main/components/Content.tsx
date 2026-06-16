import { Volume, VolumeXmark, Copy } from "@gravity-ui/icons";
import { TransResultTypes, UsualDict, Voice } from "../types/transResult";
import Loading from "./Loading";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "@heroui/react";
import { useEffect, useRef, useState } from "react";
import { getPhoneticUrl } from "../utils/tool";

const AudioPlayer = ({
    voice,
    isShow,
    isAutoPlay,
    isUs,
    word,
}: {
    voice: Voice | string;
    isShow: boolean;
    isAutoPlay: boolean;
    word: string;
    isUs: boolean;
}) => {
    const [isPlaying, setIsPlaying] = useState(false);
    const playTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    if (!isShow) return null;

    if (
        typeof voice === "string" ||
        voice.phonetic.length == 0 ||
        voice?.phonetic === undefined
    )
        return (
            <div className="flex justify-between items-center">
                <div
                    className="flex gap-1.5 font-sans font-semibold  items-center text-xs transition-transform
                text-tagSecondW justify-center h-6 px-2 rounded-md bg-tagBgW border border-borderMainW"
                >
                    <p className="pb-px">···</p>
                </div>
                <div
                    className="flex items-center justify-center w-6 h-6 rounded-md bg-tagBgW border border-borderMainW
                                   cursor-pointer transition-transform active:scale-90"
                >
                    <VolumeXmark color="#aaaaaa" width={14} height={14} />
                </div>
            </div>
        );
    const url = getPhoneticUrl(voice.phonetic, isUs, word);

    const type = isUs ? "usa" : "uk";

    const phoneticText = voice.phonetic.find(
        (item) => item.type === type,
    )?.text;

    const togglePlay = async () => {
        if (url === "") return;
        // 清除之前的定时器
        if (playTimerRef.current) clearTimeout(playTimerRef.current);
        setIsPlaying(true);
        invoke("play_phonetic_url", { url })
            .then(() => {
                const baseDuration = Math.max(word.length * 90 + 400, 800);

                const estimatedDuration = Math.round(baseDuration / 500) * 500;

                playTimerRef.current = setTimeout(
                    () => setIsPlaying(false),
                    estimatedDuration,
                );
            })
            .catch((err) => {
                toast.danger("播放音频失败！", {
                    timeout: 2000,
                });
                setIsPlaying(false);
                console.log(err);
            });
    };

    // 组件卸载时清除定时器
    useEffect(() => {
        return () => {
            if (playTimerRef.current) clearTimeout(playTimerRef.current);
        };
    }, []);

    useEffect(() => {
        if (isAutoPlay) togglePlay();
    }, [isAutoPlay]);

    return (
        <div className="flex justify-between items-center">
            <div
                className="flex gap-1.5 font-sans font-semibold  items-center text-xs transition-transform
            text-tagSecondW justify-center h-6 px-2 rounded-md bg-tagBgW border border-borderMainW"
            >
                <p className="pb-px">{phoneticText}</p>
            </div>
            <div
                onClick={togglePlay}
                className={`flex items-center justify-center w-6 h-6 rounded-md border
                               cursor-pointer transition-transform active:scale-90
                            ${
                                isPlaying
                                    ? "bg-blue-50 border-blue-200"
                                    : "bg-tagBgW border-borderMainW"
                            }`}
            >
                {url === "" ? (
                    <VolumeXmark
                        color="
                #aaaaaa"
                        width={14}
                        height={14}
                    />
                ) : isPlaying ? (
                    <div className="flex items-end justify-between w-3.5 h-3 gap-0.5">
                        <span
                            className="w-0.75 h-full rounded-full origin-bottom animate-wave-bar-1"
                            style={{ backgroundColor: "#4a90d9" }}
                        />
                        <span
                            className="w-0.75 h-full rounded-full origin-bottom animate-wave-bar-2"
                            style={{ backgroundColor: "#4a90d9" }}
                        />
                        <span
                            className="w-0.75 h-full rounded-full origin-bottom animate-wave-bar-3"
                            style={{ backgroundColor: "#4a90d9" }}
                        />
                    </div>
                ) : (
                    <Volume
                        color="
                #aaaaaa"
                        width={14}
                        height={14}
                    />
                )}
            </div>
        </div>
    );
};

function Content({
    transResult,
    isAutoPlay,
    isUs,
}: {
    transResult: TransResultTypes | null;
    isAutoPlay: boolean;
    isUs: boolean;
}) {
    if (transResult === null) {
        return <Loading />;
    }

    const { wordCard, voice, translate } = transResult.data;

    const posStyles: Record<string, string> = {
        "n.": "bg-blue-50 border border-blue-200 text-blue-600",
        "v.": "bg-green-50 border border-green-200 text-green-600",
        "adj.": "bg-purple-50 border border-purple-200 text-purple-600",
        "adv.": "bg-amber-50 border border-amber-200 text-amber-600",
    };

    const getPosStyle = (pos: string) =>
        posStyles[pos] ?? "bg-gray-100 border border-gray-200 text-gray-500";

    const UsualDict = ({ dict }: { dict: UsualDict[] }) => (
        <div className="border-t border-borderSubW pt-1 mt-2 flex flex-1 min-h-0 flex-col gap-2 overflow-y-auto no-scrollbar">
            {dict.map((entry) => (
                <div key={entry.pos} className="shrink-0">
                    {" "}
                    <span
                        className={`inline-block text-[10px] font-semibold px-1.5 pb-0.5  rounded mb-1.5 ${getPosStyle(entry.pos)}`}
                    >
                        {entry.pos}
                    </span>
                    <div className="flex flex-wrap gap-1">
                        {entry.values[0]
                            .split("；")
                            .slice(0, 10)
                            .map((val, index) => (
                                <span
                                    key={index}
                                    className="bg-gray-50 border pt-1 border-gray-200 rounded text-[11px] text-gray-600 px-1.5 py-0.5"
                                >
                                    {val}
                                </span>
                            ))}
                    </div>
                </div>
            ))}
        </div>
    );

    const handleCopy = async () => {
        try {
            await navigator.clipboard.writeText(translate.text);
            toast.success("复制成功!", {
                timeout: 1500,
            });
        } catch (err) {
            toast.danger("复制失败!", {
                timeout: 1500,
            });
        }
    };

    return (
        <div className="mx-3 flex flex-col p-3 pt-2.5 flex-1 min-h-0 border-borderMainW border rounded-xl">
            <main className="flex flex-col gap-2 ">
                <div className="flex justify-between items-center">
                    <p className="text-xl text-mainTitleW">{translate.text}</p>
                    <div
                        onClick={handleCopy}
                        className={`
                            flex items-center justify-center w-6 h-6 rounded-md border cursor-pointer
                            transition-all duration-200 active:scale-90 bg-blue-50 border-blue-200

                          `}
                    >
                        <div className={"transition-transform duration-300"}>
                            <Copy
                                // 根据状态切换颜色
                                color={"#4a90d9"}
                                width={14}
                                height={14}
                            />
                        </div>
                    </div>
                </div>

                <AudioPlayer
                    voice={voice}
                    isShow={wordCard.show}
                    isAutoPlay={isAutoPlay}
                    isUs={isUs}
                    word={translate.text}
                />

                <div className="border-t border-borderSubW my-1"></div>

                {/*常用释义*/}
                <div className="flex flex-col mt-0.5">
                    <p className="text-[10px] text-tagW font-sans">
                        {wordCard.show ? "常用" : "中文"}释义
                    </p>
                    <p className="text-xl text-mainBlueW">{translate.dit}</p>
                </div>
            </main>

            {wordCard.show && <UsualDict dict={wordCard.usualDict} />}
        </div>
    );
}

export default Content;
