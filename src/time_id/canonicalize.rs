// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::borrow::Cow;

#[derive(Debug)]
/// Canonicalization errors
pub enum CanonicalizationError {
    /// The character isn't allowed.
    UnsupportedCharacter(char),
    /// The character may not be repeated.
    UnsupportedRepetition(char),
    /// The character may not be at the start or end.
    UnsupportedPrefixOrSuffix(char),
}

/// Convert the specified name into a canonical version in order to prevent spoof IDs.
pub fn canonicalize(name: &str) -> Result<Cow<'_, str>, CanonicalizationError> {
    let mut ret = String::new();
    let mut last = None;
    let mut last_canonicalized = None;
    let mut canonical_repetitions = 0;
    let char_count = name.chars().count();
    for (i, c) in name.chars().enumerate() {
        let first_or_last = i == 0 || i == char_count - 1;
        let repeated = last == Some(c);
        if first_or_last && matches!(c, ' ') {
            return Err(CanonicalizationError::UnsupportedPrefixOrSuffix(c));
        } else if repeated && matches!(c, ' ' | '\'' | '\"') {
            return Err(CanonicalizationError::UnsupportedRepetition(c));
        } else {
            match canonicalize_char(c) {
                CanonicalizedChar::Canonical(cc) => {
                    if last_canonicalized == Some(cc) {
                        canonical_repetitions += 1;
                    } else {
                        canonical_repetitions = 0;
                    }
                    if canonical_repetitions < 2 {
                        ret.push(cc);
                    }
                    last_canonicalized = Some(cc);
                }
                CanonicalizedChar::Strip => {}
                CanonicalizedChar::Invalid => {
                    return Err(CanonicalizationError::UnsupportedCharacter(c));
                }
            }
        }
        last = Some(c);
    }
    Ok(Cow::Owned(ret))
}

enum CanonicalizedChar {
    Canonical(char),
    Strip,
    Invalid,
}

fn canonicalize_char(c: char) -> CanonicalizedChar {
    CanonicalizedChar::Canonical(match c {
        // Lowercase ASCII.
        'a'..='z' => c,
        // Uppercase ASCII.
        'A'..='H' | 'J'..='Z' => c.to_ascii_lowercase(),
        'I' => 'l',
        // Digits.
        '0' => 'o',
        '1' => 'l',
        '2' => 'z',
        '3'..='4' | '6'..='9' => c,
        '5' => 's',
        // CJK.
        //'\u{4E00}'..='\u{9fff}' => c,
        //'\u{30A0}'..='\u{30FF}' => c,
        //'\u{3040}'..='\u{3096}' | '\u{309D}'..='\u{309F}' => c,
        // Emoji.
        //'\u{1F600}'..='\u{1F64F}' => c,
        //'\u{1F900}'..='\u{1F9FF}' => c,
        //'\u{1F300}'..='\u{1F5FF}' => c,
        // Special.
        ' ' | '_' | '=' | '-' | '~' | '.' | ',' | '!' | '?' | ':' | ';' | '\'' | '"' | '#'
        | '&' => return CanonicalizedChar::Strip,
        '+' => 't',
        '$' => 's',
        _ => return CanonicalizedChar::Invalid,
    })
}

#[cfg(test)]
mod tests {
    use super::canonicalize;
    use std::collections::HashSet;

    #[test]
    fn lookalike() {
        assert_eq!(canonicalize("he1I1Io").unwrap(), "hello");
    }

    #[test]
    fn special() {
        assert_eq!(canonicalize("x_buddy_x").unwrap(), "xbuddyx");
    }

    #[test]
    fn repetitions() {
        assert_eq!(canonicalize("fod").unwrap(), "fod");
        assert_eq!(canonicalize("food").unwrap(), "food");
        assert_eq!(canonicalize("fooood").unwrap(), "food");
    }

    #[test]
    fn common() {
        #[rustfmt::skip]
        let popular = ["Doggo", "Guest", "Wulala", "hybrid", "大井抹茶", "CosmicChaos", "Dont kill me", "~ˊ˂Jsk˃", "Midway", "Admiral_Kirk", "Kirov", "!鈍器っ", "语晨情谊", "Auvergne", "Katana", "きりしま", "jr", "福州", "aaaaaaa", "Nickname", "辽宁舰", "whenthe", "Ticonderoga", "PoTaTo", "吃个桃桃", "404", "中国万岁", "Mc 幻影", "sky", "Tyyppi", "SoMeBoDy", "kr", ":D", "Somebody", "香取", "rtrouge", "蒼龍", "Fuhrer", "Assassin", "Kugelblitz", "駆逐艦", "比叡", "黃海艦長", "Rust Bucket", "Lepanto", "大和", "Arizona", "mootle", "anh cuaa", "陸奥", "Thanos", "Asylum", "Winter", "OrcaMorph", "Capt. Grey", "Ƥolyarny", "nix", "EDI", "Kirvo", "haiiiiiiiiii", "北京舰", "Itsme", "Alabama", "zero", "Miasma", "NotEDI", "Zona", "NoT_Soup", "戦艦大和", "Rankaisija", "nashe", "Bismarck", "防不了沉", "Kaiser Mumm", "ธฎ?", "Nick", "Tchaikovsky", "日本の船", "龙吟", "大井お茶", "敷島", "=PLA=", "Grey wolf", "Lighter", "ICHE", "宝格丽隆", "Firestar", "猫头刺客", "BenBWZ", "Mk.FRG4", "飛龍", "****yeah", "芙拉", "JAKI", "超大和型", "秦昭襄王", "KFC IS FIRE", "flow", "SEA SHEPHERD", "California", "GuguFish", "猫の人", "USS Yorktown", "cheems", "ITALY", "ninh:>", "Bot", "Hugo", "福建舰", "哈哈嗨", "SS独兵", "anh 2k8", "Roth", "A_girl", "SEA SHEPERD", "Blocker", "Boss", "武蔵", "万有引力", "bruhhh", "pew pew haha", "-=NIMO=-", "Enterprise", "BSCO", "たいふー", "Gugu Fish", "第一秩序", "南海舰长", "Rusiru", "电饼铛", "xWinter_Wind", "Rossiya", "fs alsasce", "Potin", "山东舰", "Nobody", "ι†smε", "THE MK48", "MC 幻影", ".", "🈸", "上海舰", "Dom Toretto", "终极规律", "Es", "中国", "Happy", "-定远舰-", "罗", "只是路过", "bjqx15", "龍鳳", "こんへ", "HMS Sparrow", "FRG4", "Son", "普莱斯", "mrawesomeッ", "Freshman", "SF DENG", "mina se olen", "生命幻象", "hung", "pearl", "华龙", "赤城", "中国红军", "TotallySoup", "嗨嗨嗨", "もかみ", "トイツ兵", "asbel 507", "!鈍器2", "孤寡", "台灣海軍", "--hodgeheg--", "栗田艦長", "IronGiant", "Vikingold", "六爷", "passer-by", "reuel", "芜湖", "Germany", "啊啊(^o^)", "Reedo", "Red October", "Navy", "青铜时代", "Luxus", "CRUSADER", "🍪Cookies", "HAIDUONG", "!鈍器", "QC酱", "阿杰", "风aza", "Polyarny", "BADAM", "Crosshair", "cjsnvjnjnvjd", "Argon^^", "1234567890", "死神", "晴嵐", "顆顆", "e", "portugal", "ƤσƖyarny", "致远舰", "Poseidon", "てて", "you died", "Sunny", "别打我", "USS Power", "Captain", "RF", "nikan", "日向", "黑天使", "Montrose", "Marvel", "东风-31B", "🔱 enutpeN", "WAR", "恒星", "Parasite", "TnT", "114514", "請加我拜", "QAQ", "San Diego", "Yomomasfatas", "Aviator", "武进号", "Tiwi", "mc1moto", "Play.com", "紀伊", "中国人", "PoTater", "Nickツ", "Crystal", "S.S. ESSESS", "Ninh:>", "极把", "うらかせ", "USS rickroll", "RSS Intrepid", "死神王者", "DJ Trump", "Sparky", "HunterKiller", "USS.ALASKA", "DreamScape", "Mk48.FRG", "A", "BattleshipTX", "蓝色空间", "🇵🇦", "-利维坦-", "鼠鼠", "Hornblower", "鸡汤", "minelayer", "Missouri", "CMG Peanut", "Doublegubble", "Snowfall", "B•B•B", "12345", "周杰伦", "∙", "MrZook", "hunggmv", "USS Enclave", "红海行动", "hiahia", "Admiral", "Neptune", "Lord Geustd", "Planet 9", "+", "Destruction", "自然选择", "防衛艦", "Tennessee43", "Oasis", "よろすや", "即", "戦艦有明", "三四四零", "dashabi", "KAKASHI", "Sub", "iulik", "iulik🦀", "214", "Miguel_777", "♡", "eeree", "初心者", "瓦良格", "USS Zumwalt", "Technoxx", "Mr. Trololo", "Black_Clover", "HaloBreaker", "MAGA川普", "RealGood", "山城", "BLACK TIGER", "中China国", "torpedo", "BISMARK", "el pro", "Astronaut", "老六", "<%Finland%>", "USAF", "斯大林", "扶桑", "雪風", "꧁H҉A҉C҉", "Blobby", "u-boat", "Peanut", "榛名", "RussianSpy81", "Nʃckn⨇mΣ", "Not Yorktown", "中国牛逼", "fuzhou", "Chu Be Dan", "BIZMARCK", "河北舰", "pop", "USS ARIZONA", "SSS", "Evil 🐿", "Thanh VN", "RSS Supreme", "至善nb", "Elliot123", "Se", "钢铁洪流", "Sneaky", "USSR", "tw 台灣", "Arvind", "USS Nimitz", "admiral K", "CRY", "THΞRMΩ", "blarg", "高級菓子", "noobenheimer", "日本帝国", "Rayden123", "Warship9001", "Pipe Bomb", "Kurapalli", "boot", "Stars", "Bartimaus", "my name", "徐", "Lozofane", "戦艦銀河", "usa", "冰晶", "Caleb2", "无畏天马", "Apocalypse", "中国人123", "CommanderMC", "a person", "Sophia;)", "BB-64", "Germany 4", "Ғˡ°ʷ", "USS Stupid", "Glitch", "☮USSR☮", "USS AKIRA", "german", "Battleship", "Technoblade", "MoistCr1TiKa", "noob", "xd", "༽Free✦", "Revenge", "Kitty", "金剛", "Yamato2199", "mrawesomeツ", "C++", "FR&G 4", "USS Arizona", "SIR YES SIR", "SU-30MKI", "藤原千花", "中国舰长", "I'm the best", "Devonshire", "pujianxin", "Lusankya", "TROY", "y", "ignorant", "Boom", "USS Wasp", "russianspy81", "Bug82", "神風", "東郷", "OrucReis", "#DENG", "Skywalker", "Beowulf", "I love cyn", "KFC", "遗迹", "哈哈哈", "三体", "uss canada", "Monmouth", "ʷ°ˡｦ", "THΣRMΩ", "123", "Orion Pax", "Putins Yatch", "⚡ZEUS", "Dreadnought", "ひさまん", "/////", "REAP3R", "抹茶", "A_GiRl", "Bubba Gump", "Server_Bot", "Շα✘", "HMS PHANTOM", "赤城にゃ", "李云龙", "QWER", "醉里寻花", "長良-wata", "RealFirestar", "北京舰长", "Capt. Zona", "USS Technoxx", "NightWind", "北大西洋", "epiphany", "Uss Arizona", "初雪绽晴", "I❤Squirrel", "Huey", "伊402", "王九日", "noob boi", "⨭FORGE⨮", "MrBeast", "RSS SUPREME", "炮哥", "IronGiant 3", "ImnotsureYT", "WhaleWars", "上海", "Noobenheimer", "samrai", "昵称", "太原舰", "湖南舰", "前有中后", "🔱 Neptune", "BuoyantBob", "いすも", "ターナ", "TU MADRE", "サニー号", "mk48.io", "第四水师", "CAL Peanut", "筑波", "Ter_Bear_101", "unix", "ARKROYALE", "打造", "！鈍器2", "Iche", "Sircheese", "Rome", "香取てす", "Autokill", "RangerFalcon", "河南舰", "Scharnhorst", "中国-Flu", "JAPAN", "CHN | Peanut", "ABW（中国", "juegagerman", "TH⨊RMΩ", "G-5 Madman", "1", "杭州舰", "中国海军", "满江红", "Pesukone", "Mexico", "Ninh2k8", "Heatwave", "Gol D roger", "LoneWolf", "Duvindu", "大明", "Dredger", "yamato", "smertz", "111中国人", "florschuetz", "**** up", "残云", "$M100,000,00", "KR0N0S", "Abyss_Runner", "サメさん", "魑魅魍魎", "bork", "Vikas", "chemical_war", "muffin head", "Shadow", "dozer", "church", "*****", "风吹叶落", "老冯", "don t kill m", "PLCTSD", "heatwave", "XD", "教主", "120", "nia", "saber", "Greyhound", "eduard", "習禁評", "共产党", "King", "Caleb", "Kreigsmarine", ":(", "我是ss k", "SUB", "Doomofall43", "Gol D Roger", "煎饼果子", "183 izumo", "Mirage", "勝者", "IronGiant 2", "饼铛", "Yamato", "Belfast", "CF-104", "Hms alliance", "ᴉosᴉlʌ", "IronGiant 1", "傻逼", "Saturn", "THE WEEEEEEE", "无名", "IJN Nagato", "ASTS-117", "SCP基金會", "2890", "YL", "...", "I forgor", "⚔", "东方舰", "まっちゃ", "Sharknemesis", "宝格丽", "空母信濃", "GamerKing", "Dodge", "Argon", "USS COLORADO", "Kriegsmarine", "<%Skaveili%>", "florentino", "#シャチ#", "Louie", "T Gaming", "Scooby doo", "炎帝", "NinjaGamer_5", "川野冷", "Polyarnyyy", "Lucifer", "python", "DUNKIRK", "die", "SinkingUsFun", "Woof", "Tchaikovksy", "Dewey", "阿差墨水", "Sgt._Clover~", "渔夫与鱼", "鈍器2", "TheGuest", "凯pro", "島風", "w(ﾟДﾟ)w", "CuO", "infinity", "profesor", "Broshop", "AboveNatural", "LaSantaMaria", "Ninh", "sss", "NARUTO", "Belgorod", "人", "吃吃吃", "guestiepoo", "夕雲", "正当防卫", "Mr. Person", "神威", "不打国人", "Alpa-ChinoPL", "PNS Ghazi", "旗艦川野", "Awesome", "葛城", "tsukuba.type", "Sebax_x >:D", "Letrixs", "k.i.o", "广州", "cool", ";D", "Big T", "meme", "Norppa", "`~<Csp>", "YAMATO", "Nightwolf", "USS SEAWOLF", "*<:@D", "Noob", "B.B.B", "初号机", "国人", "anh mcvn", "ikun", "SS", "KAI王", "・ω・", "BismarcBrakr", "R1", "青岛舰", "Shooter2", "お菓子や", "Penguin", "響", "とろろ", "天下統一", "Thunderchild", "giga chad 92", "phoenixManCZ", "bismarcbrakr", "<ZZ> GUNSDAN", "🔔", "浴火凤凰", "aaa（国人", "Absorber", "発狂抹茶", "天皇陛下", "ʷ˳ˡｦ", "武蔵＠", "白", "USS Montana", "Styer", "Uranus", ":))", "Goose AF", "Admiral doge", "かーひ", "guestはろ", "fard", "CPU", "HMS GHOST", "DISHECK", "海王", "朝風", "IG peanut_cr", "雨季的魚", "MEE", "Ninja kitty", "志愿军", "🔻", "lol", "Figure", "晴嵐零型", "お茶ァァ", "General", "Peanut✨™", "Olaf Schwolz", "XR_VorteX", "CCCP", "SSBN 727", "Soldier", "heimat", "EDIIIIIIIIII", "goodest", "🐿", "sas", "GingerT", "GHOST", "定远", "Duck", "町会長", "hi", "紀伊2", "SUSHI", "假福建舰", "Catto", "喵芝选", "お茶！", "in the end", "wyyl", "old e", "蒙古海军", "yyyyyy", "STEEL RAIN", "big mac", "歼星舰", "csp", "HUNGARIAN", "Sha-chan", ":)", "云行", "ALEX", "USS Seawolf", "东南一鸟", "Plouf", "Aurora", "YSH.cn", "Lone Wolf", "我爱华夏", "055", "thor", "Moxxie@MN", "アサホト", "Bucket", "Silver Sword", "PAIN", "筑摩", "维希法国", "MQ-9", "小羊号", "解放军", "🔱SAID🔱", "Hunter", "猎户座", "midway", "護衛艦ゆ", "PeanutCR✨", "機雷抹茶", "大东", "TRB Peanut", "-=(MARINE)=-", "edi", "广州粉丝", "QC", "Wittær", "viet nam", "IronGiant2", "Halo3rat", "Victory", "MIG", "Tardigrade", "Stealth", "guestiepoo#2", "alone", "BLACK DAGGER", "自己人", "ur a 🍋", "中山舰", "MIA KHALIFA", "Hi", "hung fake", "苏维埃级", "远望号", "追梦", "POTATO 2", "************", "SomeDude:D", "oi.84km", "adasaddas", "SF FOREVER", "ark", "CRAZY GAMES", "Lynks", "失败的man", "I want a gf", "Ivak", "整合运动", "Jack", "Sky", "Chemical_War", ":))))", "linhk7", "TNND", "语晨", "YO'PAPA", "USS SSAEATER", "c++", "劳工万岁", "Doggy", "澪", "PBullDog333", "saul goodman", "Mk54.io", "ABS", "reaper", "kota", "企业号", "StinkyWinky", "iRobot", "The Ravager", "weaaal", "EEEEEEEK!", "Iowa", "鸡汤来了", "しらぬい", "墨间花", "HMS Victory", "shark", "mikazuki", "我太男了", "唐", "Modi 🚩", "佐菲队长", "？", "MrVelocitYT", "未央公约", "🇯🇵", "SSGN 726", "呦西", "andrew", "天龙人", "Licorice", "Bruh", "LJA南海", "SERVER_BOT", "ʷ𐤏ˡｦ", "Camo?", "你干吗！", "UCraneDealer", "💘", "水母触角", "USS Texas", "Nautilus", "风吹落叶", "USS.Nevada", "西内！", "Captain Jace", "SS-5", ">El Pɘpe<", "DevFriend", "uss Arizona", "guest", "富士", "IJNお茶", "_bnacin_", "Pham T Dung", "Clemenceau", "DiamondSword", "nickname", "Scarecrow", "NASHEEE", "Submarine", "ʷ ˡｦ", "Admiralsquid", "mk48titanic", "ZETA", "💩", "INDIA", "PutinsYatch", "Gunner", "QuietClover.", "003福建舰", "peper", "是同志！", "MentatLeto", "DEADLY", "Howdy", "Ascend", "bob", "Pluto", "－＝PLA＝", "\"シャチ\"", "fireShip", "嘟", "ɴatꜱᴏc", "風林火山", "Scarnhorst", "Heisenberg", "Leon", "(^-^)", "wind", "猜我是谁", "Sorrento", "Privateer", "BEAST MODE", "辽宁", "USS REEDO", "Su-30MKI", "HARDEI", "doflamingo", "第一中中", "电锯人", "自氢", "Ken1", "時曉", "STP-星军", "AE86", "fras", "新华舰", "SpaceSoldier", "启", "okbro", "compas", "Mr.Scotty", "別打我", "IG99", "Jake", "Gloucester", "Su-30SM", "TROY/Rome", "2", "特攻隊長", "鱼头Siri", "Adam", "Elysium", "AK74m", "MIDWAY_2", "东方红", "British Lad", "Huh?", "中国红", "波動帝国", "666", "New DungUSA", "KM BISMARK", "极限号", "11451411", "織田信長", "北洋战舰", "Iowa Class", "SB", "自作戦艦", "K F C", "KM BISMARCK", "jsk", "山本7×8", "Thrawn", "剑", "AdmiralSquid", "台灣土狗", "Esteban", "嗨害嗨", "秦皇汉武", "ChildDealer", "抹茶ん", "ʷ⸰ˡｦ", "Vu M Hung", "宁远舰", "海港", "khoa", "CN    GOD", "凡人追寻", "Letriﾒs", "Wing_Air", "Clover", "七二七", "VIETNAM", "抹茶1" ];

        let mut ok = 0;
        let mut unique = HashSet::new();
        for name in popular {
            let result = canonicalize(name);
            if let Ok(result) = &result {
                ok += 1;
                if !unique.insert(result.to_owned()) {
                    print!("DUPLICATE ");
                }
            }
            println!("{name:12} -> {result:?}");
        }
        println!("{ok} out of {} ok, {} unique", popular.len(), unique.len());
    }
}
