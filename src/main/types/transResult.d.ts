export interface Translate {
    zly: string;
    errorCode: string;
    qc_type: string;
    index: string;
    from: string;
    source: string;
    text: string;
    to: string;
    id: string;
    dit: string;
    orig_text: string;
    md5: string;
}

export interface WordCard {
    title: string;
    show: boolean;
    usualDict: UsualDict[];
    secondQuery: SecondQuery[] | string;
    exchange: Exchange;
    levelList: string[];
}

export interface SecondQuery {
    k: string;
    v: string;
}

export interface UsualDict {
    values: string[];
    pos: string;
}

export interface Exchange {
    word_third?: string[];
    word_ing?: string[];
    word_done?: string[];
    word_pl?: string[];
    word_past?: string[];
    word_proto?: string[];
}

export interface Voice {
    phonetic: Phonetic[];
}

export interface Phonetic {
    text: string;
    type: string;
    filename: string;
}

export interface TransData {
    translate: Translate;
    wordCard: WordCard;
    voice: Voice | string;
}

export interface TransResultTypes {
    status: number;
    data: TransData;
    msg: string;
}

export interface TransDataZH {
    translate: Translate;
    wordCard: WordCardZH;
    voice: Voice | string;
}

export interface TransResultZHTypes {
    status: number;
    data: TransDataZH;
    msg: string;
}

export interface WordCardZH {
    title: string;
    show: boolean;
    usualDict: UsualDictZH[];
    secondQuery: SecondQueryZH[] | string;
    exchange: string;
}

export interface SecondQueryZH {
    k: string;
    v: string;
}

export interface UsualDictZH {
    values: string[];
    pos: string;
}
