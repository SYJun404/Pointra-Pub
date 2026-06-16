import { SecondQueryZH } from "../types/transResult";
import { useNavigate } from "react-router-dom";

const posStyles: Record<string, string> = {
    "n.": "bg-blue-50 border border-blue-200 text-blue-600",
    "v.": "bg-green-50 border border-green-200 text-green-600",
    "adj.": "bg-purple-50 border border-purple-200 text-purple-600",
    "adv.": "bg-amber-50 border border-amber-200 text-amber-600",
};

function parseEntry(v: string): { pos: string; meanings: string[] } {
    // 1. 匹配词性
    const match = v.match(/^([a-zA-Z./]+\.)\s*(.+)$/);
    let pos = "";
    let rawMeanings = v;

    if (match) {
        pos = match[1];
        rawMeanings = match[2];
    }

    // 2. 拆分释义
    const meaningList = rawMeanings
        .split("；")
        .map((s) => s.trim())
        .filter(Boolean);

    // ================= 宽度计算参数 =================
    const CONTAINER_WIDTH = 250; // 容器总宽度
    const CHAR_WIDTH = 11; // 每个中文汉字宽度 11px
    const ITEM_OVERHEAD = 14; // px-1.5 (12px) + border (2px)
    const GAP = 4; // item 之间的间距

    // 3. 动态计算可容纳的 item 数量
    let currentTotalWidth = 0;
    let sliceIndex = 0;

    for (let i = 0; i < meaningList.length; i++) {
        // 每个 item 的宽度 = 字数 * 11px + 14px
        const itemWidth = meaningList[i].length * CHAR_WIDTH + ITEM_OVERHEAD;

        // 第一个元素不需要加 gap，后续元素需要加上 gap
        const addedWidth = i === 0 ? itemWidth : GAP + itemWidth;

        if (currentTotalWidth + addedWidth > CONTAINER_WIDTH) {
            // 超过可用宽度，截断并确保至少保留 1 个
            sliceIndex = Math.max(i, 1);
            break;
        }

        currentTotalWidth += addedWidth;
        sliceIndex = i + 1;
    }

    return {
        pos,
        meanings: meaningList.slice(0, sliceIndex),
    };
}

function getPosStyle(pos: string) {
    return posStyles[pos] ?? "bg-gray-100 border border-gray-200 text-gray-500";
}

function SecondQueryCard({
    item,
    isLast,
    onClick,
}: {
    item: SecondQueryZH;
    isLast: boolean;
    onClick: (word: string) => void;
}) {
    const { pos, meanings } = parseEntry(item.v);

    return (
        <div
            className={`p-3 flex flex-col gap-2 transition-all duration-300 hover:bg-subBgW
                ${isLast ? "" : "border-b border-borderSubW"}`}
        >
            <div className="flex items-baseline justify-between">
                <span
                    className="text-sm font-medium text-mainBlueW cursor-pointer"
                    onClick={() => onClick(item.k)}
                >
                    {item.k}
                </span>
                {pos && (
                    <span
                        className={`text-[10px] font-semibold px-1.5 py-0.5 rounded ${getPosStyle(pos)}`}
                    >
                        {pos}
                    </span>
                )}
            </div>
            <div className="flex flex-wrap gap-1">
                {meanings.map((m) => (
                    <span
                        key={m}
                        className="bg-gray-50 border border-gray-200 rounded text-[11px] text-gray-600 px-1.5 py-0.5"
                    >
                        {m}
                    </span>
                ))}
            </div>
        </div>
    );
}

function SearchContent({ results }: { results: SecondQueryZH[] }) {
    const navigate = useNavigate();

    const handleClick = (word: string) => {
        navigate("/", { state: { word: word } });
    };
    return (
        <div className="flex flex-col">
            {results.map((item, index) => (
                <SecondQueryCard
                    key={index}
                    item={item}
                    isLast={index === results.length - 1}
                    onClick={handleClick}
                />
            ))}
        </div>
    );
}

export default SearchContent;
