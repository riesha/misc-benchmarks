use criterion::{criterion_group, criterion_main, Criterion};
use rosu_pp::OsuPP;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Score
{
    pub id:               i64,
    pub uuid:             String,
    pub map_md5:          String,
    pub score:            i64,
    pub pp:               f32,
    pub stars_total:      f32,
    pub stars_aim:        f32,
    pub stars_speed:      f32,
    pub flags:            i32,
    pub grade:            String,
    pub accuracy:         f32,
    pub ur:               f32,
    pub combo:            i16,
    pub mods:             i32,
    pub exp_mods:         i32,
    pub count_300:        i16,
    pub count_100:        i16,
    pub count_50:         i16,
    pub count_sb:         i16,
    pub count_miss:       i16,
    pub count_geki:       i16,
    pub count_katu:       i16,
    pub speed_multiplier: f32,
    pub cs:               f32,
    pub ar:               f32,
    pub od:               f32,
    pub hp:               f32,
    pub max_combo:        i32,
    pub status:           i16,

    pub online_checksum: Option<String>,
    pub perfect:         bool,
    //#[serde(skip_serializing)]
    pub updated_at:      String,
}

fn map_difficulty_range(scaled_diff: f32, min: f32, mid: f32, max: f32) -> f32
{
    if scaled_diff > 5.0
    {
        return mid + (max - mid) * (scaled_diff - 5.0) / 5.0;
    }

    if scaled_diff < 5.0
    {
        return mid - (mid - min) * (5.0 - scaled_diff) / 5.0;
    }

    mid
}

fn map_difficulty_range_inv(val: f32, min: f32, mid: f32, max: f32) -> f32
{
    if val < mid
    {
        return ((val * 5.0 - mid * 5.0) / (max - mid)) + 5.0;
    }

    if val > mid
    {
        return 5.0 - ((mid * 5.0 - val * 5.0) / (mid - min));
    }

    5.0
}

fn get_raw_approach_rate_for_speed_multiplier(approach_time: f32, speed_multiplier: f32) -> f32
{
    map_difficulty_range_inv(
        approach_time / (1.0 / speed_multiplier),
        1800.0,
        1200.0,
        450.0,
    )
}
fn get_raw_approach_time(ar: f32) -> f32 { map_difficulty_range(ar, 1800.0, 1200.0, 450.0) }
fn get_raw_overall_difficulty_for_speed_multiplier(hit_window300: f32, speed_multiplier: f32)
    -> f32
{
    map_difficulty_range_inv(hit_window300 / (1.0 / speed_multiplier), 80.0, 50.0, 20.0)
}
fn get_raw_hit_window300(od: f32) -> f32 { map_difficulty_range(od, 80.0, 50.0, 20.0) }

pub async fn calc_full(score: Score) -> Score
{
    let dir = "test.osu".to_string();

    let file = tokio::fs::read(&dir).await.unwrap();
    let mut beatmap = rosu_pp::Beatmap::parse(file.as_slice()).unwrap();

    beatmap.cs = score.cs;
    beatmap.ar = get_raw_approach_rate_for_speed_multiplier(
        get_raw_approach_time(score.ar),
        score.speed_multiplier,
    );
    beatmap.od = get_raw_overall_difficulty_for_speed_multiplier(
        get_raw_hit_window300(score.od),
        score.speed_multiplier,
    );
    beatmap.hp = score.hp;
    let res = OsuPP::new(&beatmap)
        .clock_rate(score.speed_multiplier.into())
        .mods(score.mods.try_into().unwrap()) // HDHR
        .combo(score.combo.try_into().unwrap())
        .n300(score.count_300.try_into().unwrap())
        .n100(score.count_100.try_into().unwrap())
        .n50(score.count_50.try_into().unwrap())
        .misses(score.count_miss.try_into().unwrap())
        .accuracy(score.accuracy.into())
        .calculate();

    // Ok((
    //     res.stars() as f32,
    //     res.difficulty.aim_strain as f32,
    //     res.difficulty.speed_strain as f32,
    //     res.pp() as f32,
    // ))
    Score {
        stars_total: res.stars() as f32,
        stars_aim: res.difficulty.aim_strain as f32,
        stars_speed: res.difficulty.speed_strain as f32,
        pp: res.pp() as f32,
        ..score
    }
}

fn criterion_benchmark(c: &mut Criterion)
{
    let score: Score = serde_json::from_str(
        r#"
    {
		"id": 247,
		"uuid": "1398e5f7-d506-488c-8140-9c46bea77fb1",
		"map_md5": "ea6f8e724a87cc12c8f72b7cf79bbfed",
		"score": 41217150,
		"pp": 902.6521,
		"stars_total": 9.095799,
		"stars_aim": 4.8980803,
		"stars_speed": 3.6561027,
		"flags": 1,
		"grade": "A",
		"accuracy": 98.52941,
		"ur": 77.14786,
		"combo": 1210,
		"mods": 72,
		"exp_mods": 72,
		"count_300": 1131,
		"count_100": 24,
		"count_50": 0,
		"count_sb": 0,
		"count_miss": 1,
		"count_geki": 253,
		"count_katu": 19,
		"speed_multiplier": 1.6,
		"cs": 4.0,
		"ar": 10.875,
		"od": 10.8125,
		"hp": 5.0,
		"max_combo": 1646,
		"status": 2,
		"perfect": false,
		"updated_at": "2022-09-18T15:26:20.386307Z"
	}"#,
    )
    .unwrap();
    c.bench_function("calculate pp/sr score", |b| {
        b.iter(|| async {
            calc_full(score.to_owned()).await;
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
