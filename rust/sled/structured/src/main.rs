// https://docs.rs/zerocopy/latest/zerocopy/
// https://docs.rs/zerocopy/latest/zerocopy/byteorder/index.html

//! This example demonstrates how to work with structured
//! keys and values without paying expensive (de)serialization
//! costs.
//!
//! The `upsert` function shows how to use structured keys and values.
//!
//! The `variable_lengths` function shows how to put a variable length
//! component in either the beginning or the end of your value.
//!
//! The `hash_join` function shows how to do some SQL-like joins.
//!
//! Running this example several times via `cargo run --example structured`
//! will initialize the count field to 0, and on subsequent runs it will
//! increment it.

use {
    byteorder::{BigEndian, LittleEndian},
    zerocopy::{byteorder::U64, AsBytes, FromBytes, FromZeroes, Ref, Unaligned, U16, U32},
};

/// Key と Value という2つの構造体はバイト表現との相互変換を可能にするために、zerocopy ライブラリを使用しています。
/// Key は2つの64ビット整数（ビッグエンディアン）
/// Value は64ビット整数（リトルエンディアン）と16バイトのバイト配列
/// db.update_and_fetch メソッドは指定されたキーに対する既存の値が存在する場合更新し、存在しない場合は新しい値を挿入する
fn upsert(db: &sled::Db) -> sled::Result<()> {
    // キーの種類にBigEndianを使用する理由は、それらが辞書順の順序を保持しアイテムを順番にイテレートする場合に適しているため
    // sledのアライメント要件はない、ここではzerocopyのU64型を使用している
    #[derive(FromZeroes, FromBytes, AsBytes, Unaligned)]
    #[repr(C)]
    struct Key {
        a: U64<BigEndian>,
        b: U64<BigEndian>,
    }

    // 値にはおそらくコストが低いLittleEndianを使用している
    #[derive(FromZeroes, FromBytes, AsBytes, Unaligned)]
    #[repr(C)]
    struct Value {
        count: U64<LittleEndian>,
        whatever: [u8; 16],
    }

    let key = Key {
        a: U64::new(21),
        b: U64::new(890),
    };

    db.update_and_fetch(key.as_bytes(), |value_opt| {
        if let Some(existing) = value_opt {
            // 書き込まれるコピーが必要です。
            // これにより、古いバージョンを目撃したかもしれない他のスレッドが、
            // ロックを取得せずに作業を続けることができます。
            // IVecは、22バイトに達するまでスタックに割り当てられます。
            let mut backing_bytes = sled::IVec::from(existing);

            // これにより、値が正しい長さとアライメントであることが確認されます
            // この場合、アライメントが必要ないため、
            // zerocopyからU64型を使用しています。
            let layout: Ref<&mut [u8], Value> =
                Ref::new_unaligned(&mut *backing_bytes).expect("bytes do not fit schema");

            // これにより、基本的なバイトを変更可能な構造化値として操作できます。
            let value: &mut Value = layout.into_mut();

            let new_count = value.count.get() + 1;

            println!("incrementing count to {}", new_count);

            value.count.set(new_count);

            Some(backing_bytes)
        } else {
            println!("setting count to 0");

            Some(sled::IVec::from(
                Value {
                    count: U64::new(0),
                    whatever: [0; 16],
                }
                .as_bytes(),
            ))
        }
    })?;

    Ok(())
}

// Cat values will be:
// favorite_number + battles_won + <home name variable bytes>
#[derive(FromZeroes, FromBytes, AsBytes, Unaligned)]
#[repr(C)]
struct CatValue {
    favorite_number: U64<LittleEndian>,
    battles_won: U64<LittleEndian>,
}

// Dog values will be:
// <home name variable bytes> + woof_count + postal_code
#[derive(FromZeroes, FromBytes, AsBytes, Unaligned)]
#[repr(C)]
struct DogValue {
    woof_count: U32<LittleEndian>,
    postal_code: U16<LittleEndian>,
}

// この関数では、可変長のデータを含むレコードをデータベースに挿入する方法を示しています。
// DogValue と CatValue という2つの構造体を定義しています。これらの構造体は、可変長データの前後に固定サイズのデータがある場合に使用されます。
// dogs と cats という2つのSledツリーを使用して、犬と猫の情報をデータベースに挿入します。これらの情報は可変長のホーム名と固定サイズのデータから成ります。
fn variable_lengths(db: &sled::Db) -> sled::Result<()> {
    // 以下では、zerocopyを使用して固定サイズのコンポーネントを挿入し、
    // 最後または最初に可変長レコードを混在させる方法を示します。

    // 下記のhash_joinの例では、可変部分を考慮してアイテムを読み取る方法を示しており、
    // zerocopy::Ref::{new_from_prefix、new_from_suffix}を使用しています。
    let dogs = db.open_tree(b"dogs")?;

    let mut dog2000_value = vec![];
    dog2000_value.extend_from_slice(b"science zone");
    dog2000_value.extend_from_slice(
        DogValue {
            woof_count: U32::new(666),
            postal_code: U16::new(42),
        }
        .as_bytes(),
    );
    dogs.insert("dog2000", dog2000_value)?;

    let mut zed_pup_value = vec![];
    zed_pup_value.extend_from_slice(b"bowling alley");
    zed_pup_value.extend_from_slice(
        DogValue {
            woof_count: U32::new(32113231),
            postal_code: U16::new(0),
        }
        .as_bytes(),
    );
    dogs.insert("zed pup", zed_pup_value)?;

    // IMPORTANT NOTE: German dogs eat food called "barf"
    let mut klaus_value = vec![];
    klaus_value.extend_from_slice(b"barf shop");
    klaus_value.extend_from_slice(
        DogValue {
            woof_count: U32::new(0),
            postal_code: U16::new(12045),
        }
        .as_bytes(),
    );
    dogs.insert("klaus", klaus_value)?;

    let cats = db.open_tree(b"cats")?;

    let mut laser_cat_value = vec![];
    laser_cat_value.extend_from_slice(
        CatValue {
            favorite_number: U64::new(11),
            battles_won: U64::new(321231321),
        }
        .as_bytes(),
    );
    laser_cat_value.extend_from_slice(b"science zone");
    cats.insert("laser cat", laser_cat_value)?;

    let mut pulsar_cat_value = vec![];
    pulsar_cat_value.extend_from_slice(
        CatValue {
            favorite_number: U64::new(11),
            battles_won: U64::new(321231321),
        }
        .as_bytes(),
    );
    pulsar_cat_value.extend_from_slice(b"science zone");
    cats.insert("pulsar cat", pulsar_cat_value)?;

    let mut fluffy_value = vec![];
    fluffy_value.extend_from_slice(
        CatValue {
            favorite_number: U64::new(11),
            battles_won: U64::new(321231321),
        }
        .as_bytes(),
    );
    fluffy_value.extend_from_slice(b"bowling alley");
    cats.insert("fluffy", fluffy_value)?;

    Ok(())
}

// この関数では、キーと値の両方に可変長データを含むデータベースのエントリを結合します。
// cats と dogs ツリーから情報を取得し、それぞれのホーム名をキーとして、猫と犬の情報を結合します。これにより、同じ家に住む猫と犬の情報をマッチングできます。
fn hash_join(db: &sled::Db) -> sled::Result<()> {
    // here we will try to find cats and dogs who
    // live in the same home.

    let cats = db.open_tree(b"cats")?;
    let dogs = db.open_tree(b"dogs")?;

    let mut join = std::collections::HashMap::new();

    for name_value_res in &cats {
        // cats are stored as name -> favorite_number + battles_won + home name
        // variable bytes
        let (name, value_bytes) = name_value_res?;
        let (_, home_name): (Ref<&[u8], CatValue>, &[u8]) =
            Ref::new_from_prefix(&*value_bytes).unwrap();
        let (ref mut cat_names, _dog_names) =
            join.entry(home_name.to_vec()).or_insert((vec![], vec![]));
        cat_names.push(std::str::from_utf8(&*name).unwrap().to_string());
    }

    for name_value_res in &dogs {
        // dogs are stored as name -> home name variable bytes + woof count +
        // postal code
        let (name, value_bytes) = name_value_res?;

        // これに注意してください。これは、先ほどのcatの例と逆です。
        // そこでは、可変バイトは値の逆の端にあり、
        // new_from_prefixを使用して抽出されます。
        let (home_name, _dog_value): (_, Ref<&[u8], DogValue>) =
            Ref::new_from_suffix(&*value_bytes).unwrap();

        if let Some((_cat_names, ref mut dog_names)) = join.get_mut(home_name) {
            dog_names.push(std::str::from_utf8(&*name).unwrap().to_string());
        }
    }

    for (home, (cats, dogs)) in join {
        println!(
            "the cats {:?} and the dogs {:?} live in the same home of {}",
            cats,
            dogs,
            std::str::from_utf8(&home).unwrap()
        );
    }

    Ok(())
}

// sled データベースを開き、上記の関数を順番に呼び出します。データベースを開き、データを更新し、可変長のデータを挿入し、最後にデータを結合します。
// これらの関数と構造体は、Sledデータベース内の構造化されたデータを効率的に操作する方法を示しており、データベースのアプリケーションで使用できるテクニックを提供しています。
fn main() -> sled::Result<()> {
    let db = sled::open("my_database")?;
    upsert(&db)?;
    variable_lengths(&db)?;
    hash_join(&db)?;

    Ok(())
}
