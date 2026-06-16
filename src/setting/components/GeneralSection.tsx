import type { GeneralSetting } from "../types";
import { Switch, Slider } from "@heroui/react";

const phonetisStyles: Record<
    string,
    { bg: string; border: string; color: string; label: string }
> = {
    us: {
        bg: "bg-purple-50",
        border: "border-purple-200",
        color: "text-[#8b5cf6]",
        label: "美式",
    },
    uk: {
        bg: "bg-cyan-50",
        border: "border-cyan-200",
        color: "text-[#06b6d4]",
        label: "英式",
    },
};

const PhoneticSwitch = ({
    onUpdate,
    value,
    id,
}: {
    id: string;
    value: string;
    onUpdate: (id: string, value: boolean | string | number) => void;
}) => {
    const cur = value === "us" ? "us" : "uk";
    const style = phonetisStyles[cur];

    const changeValue = () => {
        const next = cur === "us" ? "uk" : "us";
        onUpdate(id, next);
    };

    return (
        <div
            onClick={changeValue}
            className={`${style.bg} ${style.border} border w-10 h-5 rounded-full flex justify-center items-center cursor-pointer`}
        >
            <p className={`text-[11px] pt-0.5 ${style.color}`}>{style.label}</p>
        </div>
    );
};

export default function GeneralSection({
    settings,
    onToggle,
    onUpdate,
}: {
    settings: GeneralSetting[];
    onToggle: (id: string) => void;
    onUpdate: (id: string, value: boolean | string | number) => void;
}) {
    return (
        <section>
            <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                    <div className="w-1 h-4 rounded-full bg-mainBlueW" />
                    <h2 className="text-sm font-medium text-mainTitleW">
                        通用设置
                    </h2>
                </div>
                <p className="text-[11px] text-tagSecondW mt-px rounded-md ">
                    General
                </p>
            </div>

            <div className="rounded-xl border border-borderMainW divide-y divide-borderSubW bg-white overflow-hidden">
                {settings.map((item) => {
                    // --- 单词发音：下拉选择器 ---
                    if (item.id === "pronunciation") {
                        return (
                            <div
                                key={item.id}
                                className="flex items-center justify-between p-3 hover:bg-subBgW transition-colors"
                            >
                                <div className="flex-1 min-w-0 mr-3">
                                    <p className="text-sm text-mainTitleW">
                                        {item.label}
                                    </p>
                                    <p className="text-[11px] text-tagSecondW mt-0.5">
                                        {item.description}
                                    </p>
                                </div>
                                <div className="relative inline-flex items-center cursor-pointer shrink-0">
                                    <PhoneticSwitch
                                        onUpdate={onUpdate}
                                        id={item.id}
                                        value={item.value as string}
                                    />
                                </div>
                            </div>
                        );
                    }

                    // --- 发音音量：滑动条 ---
                    if (item.id === "pronunciation_volume") {
                        const volume = item.value as number;
                        return (
                            <div
                                key={item.id}
                                className="flex items-center justify-between p-3 hover:bg-subBgW transition-colors"
                            >
                                <div className="flex-1 min-w-0 mr-3">
                                    <p className="text-sm text-mainTitleW">
                                        {item.label}
                                    </p>
                                    <p className="text-[11px] text-tagSecondW mt-0.5">
                                        {item.description}
                                    </p>
                                </div>
                                <div className="relative inline-flex items-center cursor-pointer shrink-0">
                                    <Slider
                                        aria-label="Slider"
                                        className="w-30"
                                        value={volume}
                                        onChange={(value) => {
                                            const num = Number(value);
                                            if (num <= 10) {
                                                onUpdate(item.id, 10);
                                                return;
                                            }
                                            onUpdate(item.id, num);
                                        }}
                                    >
                                        <Slider.Output />
                                        <Slider.Track>
                                            <Slider.Fill />
                                            <Slider.Thumb />
                                        </Slider.Track>
                                    </Slider>
                                </div>
                            </div>
                        );
                    }

                    // --- 默认：开关切换 ---
                    return (
                        <div
                            key={item.id}
                            className="flex items-center justify-between p-3 hover:bg-subBgW transition-colors"
                        >
                            <div className="flex-1 min-w-0 mr-3">
                                <p className="text-sm text-mainTitleW">
                                    {item.label}
                                </p>
                                <p className="text-[11px] text-tagSecondW mt-0.5">
                                    {item.description}
                                </p>
                            </div>
                            <div className="relative inline-flex items-center cursor-pointer shrink-0">
                                <Switch
                                    isSelected={item.value as boolean}
                                    onChange={() => onToggle(item.id)}
                                >
                                    <Switch.Control>
                                        <Switch.Thumb />
                                    </Switch.Control>
                                </Switch>
                            </div>
                        </div>
                    );
                })}
            </div>
        </section>
    );
}
