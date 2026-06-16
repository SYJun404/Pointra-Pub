import { Phonetic } from "../types/transResult";

export function judgeSentence(str: string) {
    const cleaned = str.trim();
    if (!cleaned) return false;

    // 1. 优先排除带有句尾标点符号，且前面有多个词的情况
    // 2. 使用正则提取单词。允许字母、单引号(don't)、连字符(well-being)
    const words = cleaned.match(/[a-zA-Z'-]+/g);

    if (!words) return true;

    // 处理特殊缩写，比如 U.S.A. 如果去掉点之后就是一个词
    if (words.length > 1) {
        // 检查是不是类似 U.S.A. 这种被点号隔开的单字缩写
        const isAbbreviation =
            words.every((w) => w.length === 1) && cleaned.includes(".");
        if (isAbbreviation) {
            return false;
        }
    }

    return words.length === 1 ? false : true;
}

export function getPhoneticUrl(
    phonetic: Phonetic[],
    isUSA: boolean,
    word: string,
) {
    const type = isUSA ? "usa" : "uk";
    const url = phonetic.find((item) => item.type === type)?.filename ?? "";
    if (url !== "") return url;
    return `https://dict.youdao.com/dictvoice?type=${!isUSA ? "1" : "0"}&audio=${word.toLowerCase()}`;
}
