// because iterators use references, and i REFUSE to use closures
#![allow(clippy::trivially_copy_pass_by_ref, clippy::module_name_repetitions)]

use std::f64::consts::{PI, TAU};

use ordered_float::OrderedFloat;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{Color, Components, I2PState, Sprite};

use self::kmeans::dither_kmeans;

use wasm_bindgen::prelude::*;

mod kmeans;

const DITHER_THRESHOLD_BAYER8X8: [f32; 64] = [
    0.0 / 64.0,
    32.0 / 64.0,
    8.0 / 64.0,
    40.0 / 64.0,
    2.0 / 64.0,
    34.0 / 64.0,
    10.0 / 64.0,
    42.0 / 64.0,
    48.0 / 64.0,
    16.0 / 64.0,
    56.0 / 64.0,
    24.0 / 64.0,
    50.0 / 64.0,
    18.0 / 64.0,
    58.0 / 64.0,
    26.0 / 64.0,
    12.0 / 64.0,
    44.0 / 64.0,
    4.0 / 64.0,
    36.0 / 64.0,
    14.0 / 64.0,
    46.0 / 64.0,
    6.0 / 64.0,
    38.0 / 64.0,
    60.0 / 64.0,
    28.0 / 64.0,
    52.0 / 64.0,
    20.0 / 64.0,
    62.0 / 64.0,
    30.0 / 64.0,
    54.0 / 64.0,
    22.0 / 64.0,
    3.0 / 64.0,
    35.0 / 64.0,
    11.0 / 64.0,
    43.0 / 64.0,
    1.0 / 64.0,
    33.0 / 64.0,
    9.0 / 64.0,
    41.0 / 64.0,
    51.0 / 64.0,
    19.0 / 64.0,
    59.0 / 64.0,
    27.0 / 64.0,
    49.0 / 64.0,
    17.0 / 64.0,
    57.0 / 64.0,
    25.0 / 64.0,
    15.0 / 64.0,
    47.0 / 64.0,
    7.0 / 64.0,
    39.0 / 64.0,
    13.0 / 64.0,
    45.0 / 64.0,
    5.0 / 64.0,
    37.0 / 64.0,
    63.0 / 64.0,
    31.0 / 64.0,
    55.0 / 64.0,
    23.0 / 64.0,
    61.0 / 64.0,
    29.0 / 64.0,
    53.0 / 64.0,
    21.0 / 64.0,
];
const DITHER_THRESHOLD_BAYER4X4: [f32; 16] = [
    0.0 / 16.0,
    8.0 / 16.0,
    2.0 / 16.0,
    10.0 / 16.0,
    12.0 / 16.0,
    4.0 / 16.0,
    14.0 / 16.0,
    6.0 / 16.0,
    3.0 / 16.0,
    11.0 / 16.0,
    1.0 / 16.0,
    9.0 / 16.0,
    15.0 / 16.0,
    7.0 / 16.0,
    13.0 / 16.0,
    5.0 / 16.0,
];
const DITHER_THRESHOLD_BAYER2X2: [f32; 4] = [0.0 / 4.0, 2.0 / 4.0, 3.0 / 4.0, 1.0 / 4.0];
const DITHER_THRESHOLD_CLUSTER8X8: [f32; 64] = [
    24.0 / 64.0,
    10.0 / 64.0,
    12.0 / 64.0,
    26.0 / 64.0,
    35.0 / 64.0,
    47.0 / 64.0,
    49.0 / 64.0,
    37.0 / 64.0,
    8.0 / 64.0,
    0.0 / 64.0,
    2.0 / 64.0,
    14.0 / 64.0,
    45.0 / 64.0,
    59.0 / 64.0,
    61.0 / 64.0,
    51.0 / 64.0,
    22.0 / 64.0,
    6.0 / 64.0,
    4.0 / 64.0,
    16.0 / 64.0,
    43.0 / 64.0,
    57.0 / 64.0,
    63.0 / 64.0,
    53.0 / 64.0,
    30.0 / 64.0,
    20.0 / 64.0,
    18.0 / 64.0,
    28.0 / 64.0,
    33.0 / 64.0,
    41.0 / 64.0,
    55.0 / 64.0,
    39.0 / 64.0,
    34.0 / 64.0,
    46.0 / 64.0,
    48.0 / 64.0,
    36.0 / 64.0,
    25.0 / 64.0,
    11.0 / 64.0,
    13.0 / 64.0,
    27.0 / 64.0,
    44.0 / 64.0,
    58.0 / 64.0,
    60.0 / 64.0,
    50.0 / 64.0,
    9.0 / 64.0,
    1.0 / 64.0,
    3.0 / 64.0,
    15.0 / 64.0,
    42.0 / 64.0,
    56.0 / 64.0,
    62.0 / 64.0,
    52.0 / 64.0,
    23.0 / 64.0,
    7.0 / 64.0,
    5.0 / 64.0,
    17.0 / 64.0,
    32.0 / 64.0,
    40.0 / 64.0,
    54.0 / 64.0,
    38.0 / 64.0,
    31.0 / 64.0,
    21.0 / 64.0,
    19.0 / 64.0,
    29.0 / 64.0,
];
const DITHER_THRESHOLD_CLUSTER4X4: [f32; 16] = [
    12.0 / 16.0,
    5.0 / 16.0,
    6.0 / 16.0,
    13.0 / 16.0,
    4.0 / 16.0,
    0.0 / 16.0,
    1.0 / 16.0,
    7.0 / 16.0,
    11.0 / 16.0,
    3.0 / 16.0,
    2.0 / 16.0,
    8.0 / 16.0,
    15.0 / 16.0,
    10.0 / 16.0,
    9.0 / 16.0,
    14.0 / 16.0,
];

type DistanceFunction = dyn Fn(&[Color], &[Components], Color) -> Color + Sync;

#[derive(Default, Clone, Copy)]
#[wasm_bindgen]
pub enum DitherMode {
    #[default]
    None,
    Bayer8x8,
    Bayer4x4,
    Bayer2x2,
    Cluster8x8,
    Cluster4x4,
    FloydComponent,
    FloydDistributed,
}

#[derive(Default, PartialEq, Clone, Copy)]
#[wasm_bindgen]
pub enum DistanceMode {
    KMeans,
    RGB,
    LWRGB,
    Redmean,
    CIE76,
    CIE94,
    CIEDE2000,
    CMC,
    XYZ,
    YCC,
    YIQ,
    YUV,
    #[default]
    OKLab,
}

pub fn dither_image(
    state: &mut I2PState,
    input: &[Color],
    output: &mut Sprite,
    width: usize,
    height: usize,
) {
    if state.dither_options.pixel_distance_mode == DistanceMode::KMeans {
        dither_kmeans(state, input, output, width, height);
        return;
    }

    let palette_components: Vec<Components> = match state.dither_options.pixel_distance_mode {
        DistanceMode::RGB | DistanceMode::LWRGB | DistanceMode::Redmean => {
            state.palette.iter().map(color_to_rgb).collect()
        }
        DistanceMode::CIE76 | DistanceMode::CIE94 | DistanceMode::CIEDE2000 | DistanceMode::CMC => {
            state.palette.iter().map(color_to_lab).collect()
        }
        DistanceMode::XYZ => state.palette.iter().map(color_to_xyz).collect(),
        DistanceMode::YCC => state.palette.iter().map(color_to_ycc).collect(),
        DistanceMode::YIQ => state.palette.iter().map(color_to_yiq).collect(),
        DistanceMode::YUV => state.palette.iter().map(color_to_yuv).collect(),
        DistanceMode::OKLab => state.palette.iter().map(color_to_oklab).collect(),
        DistanceMode::KMeans => unreachable!(),
    };

    let find_closest = match state.dither_options.pixel_distance_mode {
        DistanceMode::RGB => palette_find_closest(color_to_rgb, color_dist2),
        DistanceMode::LWRGB => palette_find_closest(color_to_rgb, lwrgb_color_dist2),
        DistanceMode::Redmean => palette_find_closest(color_to_rgb, redmean_color_dist2),
        DistanceMode::CIE76 => palette_find_closest(color_to_lab, color_dist2),
        DistanceMode::CIE94 => palette_find_closest(color_to_lab, cie94_color_dist2),
        DistanceMode::CIEDE2000 => palette_find_closest(color_to_lab, ciede2000_color_dist2),
        DistanceMode::CMC => palette_find_closest(color_to_lab, cmc_color_dist2),
        DistanceMode::XYZ => palette_find_closest(color_to_xyz, color_dist2),
        DistanceMode::YCC => palette_find_closest(color_to_ycc, color_dist2),
        DistanceMode::YIQ => palette_find_closest(color_to_yiq, color_dist2),
        DistanceMode::YUV => palette_find_closest(color_to_yuv, color_dist2),
        DistanceMode::OKLab => palette_find_closest(color_to_oklab, color_dist2),
        DistanceMode::KMeans => unreachable!(),
    };

    match state.dither_options.pixel_dither_mode {
        DitherMode::None => dither_none(
            state,
            input,
            &mut output.data,
            &state.palette,
            &palette_components,
            find_closest,
        ),
        DitherMode::Bayer8x8 => dither_threshold(
            state,
            input,
            &mut output.data,
            &state.palette,
            &palette_components,
            find_closest,
            width,
            &DITHER_THRESHOLD_BAYER8X8,
            3,
        ),
        DitherMode::Bayer4x4 => dither_threshold(
            state,
            input,
            &mut output.data,
            &state.palette,
            &palette_components,
            find_closest,
            width,
            &DITHER_THRESHOLD_BAYER4X4,
            2,
        ),
        DitherMode::Bayer2x2 => dither_threshold(
            state,
            input,
            &mut output.data,
            &state.palette,
            &palette_components,
            find_closest,
            width,
            &DITHER_THRESHOLD_BAYER2X2,
            1,
        ),
        DitherMode::Cluster8x8 => dither_threshold(
            state,
            input,
            &mut output.data,
            &state.palette,
            &palette_components,
            find_closest,
            width,
            &DITHER_THRESHOLD_CLUSTER8X8,
            3,
        ),
        DitherMode::Cluster4x4 => dither_threshold(
            state,
            input,
            &mut output.data,
            &state.palette,
            &palette_components,
            find_closest,
            width,
            &DITHER_THRESHOLD_CLUSTER4X4,
            2,
        ),
        DitherMode::FloydComponent => todo!(),
        DitherMode::FloydDistributed => todo!(),
    }
}

fn palette_find_closest(
    conversion: impl Fn(&Color) -> Components + 'static + Sync,
    distance: impl Fn(&Components, &Components) -> f64 + 'static + Sync,
) -> Box<DistanceFunction> {
    Box::new(
        move |palette: &[Color], palette_components: &[Components], color: Color| {
            if color.alpha == 0 {
                return palette[0];
            }

            let input = conversion(&color);

            let index = palette_components
                .iter()
                .enumerate()
                .min_by_key(|(_, c)| OrderedFloat(distance(&input, c)))
                .map_or(0, |(i, _)| i);

            palette[index]
        },
    )
}

fn color_dist2(a: &Components, b: &Components) -> f64 {
    let diff_0 = b.0 - a.0;
    let diff_1 = b.1 - a.1;
    let diff_2 = b.2 - a.2;

    diff_0 * diff_0 + diff_1 * diff_1 + diff_2 * diff_2
}

fn lwrgb_color_dist2(a: &Components, b: &Components) -> f64 {
    let dr = b.0 - a.0;
    let dg = b.1 - a.1;
    let db = b.2 - a.2;
    let l1 = a.0 * 0.299 + a.1 * 0.587 + a.2 * 0.114;
    let l2 = b.0 * 0.299 + b.1 * 0.587 + b.2 * 0.114;
    let dl = l1 - l2;

    (dr * dr * 0.299 + dg * dg * 0.587 + db * db * 0.114) * 0.75 + dl * dl
}

fn redmean_color_dist2(a: &Components, b: &Components) -> f64 {
    let rmean = (a.0 + b.0) / 2.0;
    let r = a.0 - b.0;
    let g = a.1 - b.1;
    let b = a.2 - b.2;

    (((512.0 + rmean) * r * r) / 256.0) + 4.0 * g * g + (((767.0 - rmean) * b * b) / 256.0)
}

fn cie94_color_dist2(col0: &Components, col1: &Components) -> f64 {
    let l = col0.0 - col1.0;
    let c1 = f64::sqrt(col0.1 * col0.1 + col0.2 * col0.2);
    let c2 = f64::sqrt(col1.1 * col1.1 + col1.2 * col1.2);
    let c = c1 - c2;
    let h = f64::sqrt(
        (col0.1 - col1.1) * (col0.1 - col1.1) + (col0.2 - col1.2) * (col0.2 - col1.2) - c * c,
    );
    let r1 = l;
    let r2 = c / (1.0 + 0.045 * c1);
    let r3 = h / (1.0 + 0.015 * c1);

    r1 * r1 + r2 * r2 + r3 * r3
}

#[allow(clippy::similar_names)]
fn ciede2000_color_dist2(col0: &Components, col1: &Components) -> f64 {
    let c1 = f64::sqrt(col0.1 * col0.1 + col0.2 * col0.2);
    let c2 = f64::sqrt(col1.1 * col1.1 + col1.2 * col1.2);
    let c_ = (c1 + c2) / 2.0;

    let c_p2 = c_.powf(7.0);
    let mut v = 0.5 * (1.0 - f64::sqrt(c_p2 / (c_p2 + 6_103_515_625.0)));
    let a1 = (1.0 + v) * col0.1;
    let a2 = (1.0 + v) * col1.1;

    let cs1 = f64::sqrt(a1 * a1 + col0.2 * col0.2);
    let cs2 = f64::sqrt(a2 * a2 + col1.2 * col1.2);

    let mut h1 = 0.0;
    if col0.2 != 0.0 || a1 != 0.0 {
        h1 = col0.2.atan2(a1);
        if h1 < 0.0 {
            h1 += TAU;
        }
    }
    let mut h2 = 0.0;
    if col1.2 != 0.0 || a2 != 0.0 {
        h2 = col1.2.atan2(a2);
        if h2 < 0.0 {
            h2 += TAU;
        }
    }

    let l = col1.0 - col0.0;
    let cs = cs2 - cs1;
    let mut h = 0.0;
    if cs1 * cs2 != 0.0 {
        h = h2 - h1;
        if h < -PI {
            h += TAU;
        } else if h > PI {
            h -= TAU;
        }
    }
    let h = 2.0 * f64::sqrt(cs1 * cs2) * f64::sin(h / 2.0);

    let cs_ = (cs1 + cs2) / 2.0;
    let mut h_ = h1 + h2;
    if cs1 * cs2 != 0.0 {
        if f64::abs(h1 - h2) <= PI {
            h_ = (h1 + h2) / 2.0;
        } else if h1 + h2 < TAU {
            h_ = (h1 + h2 + TAU) / 2.0;
        } else {
            h_ = (h1 + h2 - TAU) / 2.0;
        }
    }

    let t = 1.0 - 0.17 * f64::cos(h_ - 30.0_f64.to_radians())
        + 0.24 * f64::cos(2.0 * h_)
        + 0.32 * f64::cos(3.0 * h_ + 6.0_f64.to_radians())
        - 0.2 * f64::cos(4.0 * h_ - 63.0_f64.to_radians());
    v = 60.0_f64.to_radians()
        * f64::exp(
            -1.0 * ((h_ - 275.0_f64.to_radians()) / 25.0_f64.to_radians())
                * ((h_ - 275.0_f64.to_radians()) / 25.0_f64.to_radians()),
        );
    let cs_p2 = cs_.powf(7.0);
    let rc = 2.0 * f64::sqrt(cs_p2 / (cs_p2 + 6_103_515_625.0));
    let rt = -1.0 * v.sin() * rc;
    let sl = 1.0;
    let sc = 1.0 + 0.045 * cs_;
    let sh = 1.0 + 0.015 * cs_ * t;

    (l / sl) * (l / sl) + (cs / sc) * (cs / sc) + (h / sh) * (h / sh) + rt * (cs / sc) * (h_ / sh)
}

fn cmc_color_dist2(col0: &Components, col1: &Components) -> f64 {
    let c1 = f64::sqrt(col0.1.powf(2.0) + col0.2.powf(2.0));
    let c2 = f64::sqrt(col1.1.powf(2.0) + col1.2.powf(2.0));
    let ff = f64::sqrt(c1.powf(4.0) / (c1.powf(4.0) + 1900.0));
    let h1: f64;
    let mut bias: f64 = 0.0;
    if col0.1 >= 0.0 && col0.2 == 0.0 {
        h1 = 0.0;
    } else if col0.1 < 0.0 && col0.2 == 0.0 {
        h1 = 180.0;
    } else if col0.1 == 0.0 && col0.2 > 0.0 {
        h1 = 90.0;
    } else if col0.1 == 0.0 && col0.2 < 0.0 {
        h1 = 270.0;
    } else {
        if col0.1 > 0.0 && col0.2 > 0.0 {
            bias = 0.0;
        }
        if col0.1 < 0.0 {
            bias = 180.0;
        }
        if col0.1 > 0.0 && col0.2 < 0.0 {
            bias = 360.0;
        }
        h1 = (col0.2 / col0.1).atan().to_degrees() + bias;
    }

    let tt = if (164.0..=345.0).contains(&h1) {
        0.56 + f64::abs(0.2 * f64::cos(168.0 + h1))
    } else {
        0.36 + f64::abs(0.4 * f64::cos(35.0 + h1))
    };

    let mut sl = if col0.0 < 16.0 {
        0.511
    } else {
        (0.040_975 * col0.0) / (1.0 + (0.01765 * col0.0))
    };

    let mut sc = ((0.0638 * c1) / (1.0 + (0.0131 * c1))) + 0.638;
    let mut sh = ((ff * tt) + 1.0 - ff) * sc;
    let dh =
        f64::sqrt((col1.1 - col0.1).powf(2.0) + (col1.2 - col0.2).powf(2.0) - (c2 - c1).powf(2.0));
    sl = (col1.0 - col0.0) / (2.0 * sl);
    sc = (c2 - c1) / (1.0 * sc);
    sh = dh / sh;
    sl.powf(2.0) + sc.powf(2.0) + sh.powf(2.0)
}

fn color_to_oklab(color: &Color) -> Components {
    let r = f64::from(color.red) / 255.0;
    let g = f64::from(color.green) / 255.0;
    let b = f64::from(color.blue) / 255.0;

    let lr = if r >= 0.04045 {
        ((r + 0.055) / 1.055).powf(2.4)
    } else {
        r / 12.92
    };
    let lg = if g >= 0.04045 {
        ((g + 0.055) / 1.055).powf(2.4)
    } else {
        g / 12.92
    };
    let lb = if b >= 0.04045 {
        ((b + 0.055) / 1.055).powf(2.4)
    } else {
        b / 12.92
    };

    let l = 0.412_221_470_8 * lr + 0.536_332_536_3 * lg + 0.051_445_992_9 * lb;
    let m = 0.211_903_498_2 * lr + 0.680_699_545_1 * lg + 0.107_396_956_6 * lb;
    let s = 0.088_302_461_9 * lr + 0.281_718_837_6 * lg + 0.629_978_700_5 * lb;

    let cbrt_l = l.cbrt();
    let cbrt_m = m.cbrt();
    let cbrt_s = s.cbrt();

    Components(
        0.210_454_255_3 * cbrt_l + 0.793_617_785_0 * cbrt_m - 0.004_072_046_8 * cbrt_s,
        1.977_998_495_1 * cbrt_l - 2.428_592_205_0 * cbrt_m + 0.450_593_709_9 * cbrt_s,
        0.025_904_037_1 * cbrt_l + 0.782_771_766_2 * cbrt_m - 0.808_675_766_0 * cbrt_s,
    )
}

fn color_to_ycc(color: &Color) -> Components {
    let r = f64::from(color.red);
    let g = f64::from(color.green);
    let b = f64::from(color.blue);

    Components(
        0.299 * r + 0.587 * g + 0.114 * b,
        -0.16874 * r - 0.33126 * g + 0.5 * b,
        0.5 * r - 0.41869 * g - 0.08131 * b,
    )
}

fn color_to_yiq(color: &Color) -> Components {
    let r = f64::from(color.red);
    let g = f64::from(color.green);
    let b = f64::from(color.blue);

    Components(
        0.2999 * r + 0.587 * g + 0.114 * b,
        0.595_716 * r - 0.274_453 * g - 0.321_264 * b,
        0.211_456 * r - 0.522_591 * g + 0.31135 * b,
    )
}

fn color_to_yuv(color: &Color) -> Components {
    let r = f64::from(color.red);
    let g = f64::from(color.green);
    let b = f64::from(color.blue);

    let c0 = 0.2999 * r + 0.587 * g + 0.114 * b;
    let c1 = 0.492 * (b - c0);
    let c2 = 0.887 * (r - c0);

    Components(c0, c1, c2)
}

//Convert to xyz then to lab color space
fn color_to_lab(color: &Color) -> Components {
    let mut xyz = color_to_xyz(color);

    //x component
    if xyz.0 > 0.008_856 {
        xyz.0 = f64::powf(xyz.0, 1.0 / 3.0);
    } else {
        xyz.0 = (7.787 * xyz.0) + (16.0 / 116.0);
    }

    //y component
    if xyz.1 > 0.008_856 {
        xyz.1 = f64::powf(xyz.1, 1.0 / 3.0);
    } else {
        xyz.1 = (7.787 * xyz.1) + (16.0 / 116.0);
    }

    //z component
    if xyz.2 > 0.008_856 {
        xyz.2 = f64::powf(xyz.2, 1.0 / 3.0);
    } else {
        xyz.2 = (7.787 * xyz.2) + (16.0 / 116.0);
    }

    Components(
        116.0 * xyz.1 - 16.0,
        500.0 * (xyz.0 - xyz.1),
        200.0 * (xyz.1 - xyz.2),
    )
}

fn color_to_xyz(color: &Color) -> Components {
    let mut input = color_to_rgb(color);

    //red component
    if input.0 > 0.04045 {
        input.0 = f64::powf((input.0 + 0.055) / 1.055, 2.4) * 100.0;
    } else {
        input.0 = (input.0 / 12.92) * 100.0;
    }

    //green component
    if input.1 > 0.04045 {
        input.1 = f64::powf((input.1 + 0.055) / 1.055, 2.4) * 100.0;
    } else {
        input.1 = (input.1 / 12.92) * 100.0;
    }

    //blue component
    if input.2 > 0.04045 {
        input.2 = f64::powf((input.2 + 0.055) / 1.055, 2.4) * 100.0;
    } else {
        input.2 = (input.2 / 12.92) * 100.0;
    }

    Components(
        (input.0 * 0.4124 + input.1 * 0.3576 + input.2 * 0.1805) / 95.05,
        (input.0 * 0.2126 + input.1 * 0.7152 + input.2 * 0.0722) / 100.0,
        (input.0 * 0.0193 + input.1 * 0.1192 + input.2 * 0.9504) / 108.89,
    )
}

fn color_to_rgb(color: &Color) -> Components {
    Components(
        f64::from(color.red) / 255.0,
        f64::from(color.green) / 255.0,
        f64::from(color.blue) / 255.0,
    )
}

fn dither_none_apply(state: &mut I2PState, input: &[Color], output: &mut [Color]) {
    for (cin, output) in input.iter().zip(output) {
        if cin.alpha < state.dither_options.alpha_threshold {
            *output = Color::default();
            continue;
        }

        *output = *cin;
        output.alpha = 255;
    }
}

fn dither_none(
    state: &I2PState,
    input: &[Color],
    output: &mut [Color],
    palette: &[Color],
    palette_components: &[Components],
    closest: impl Fn(&[Color], &[Components], Color) -> Color,
) {
    for (cin, output) in input.iter().zip(output) {
        if cin.alpha < state.dither_options.alpha_threshold {
            *output = Color::default();
            continue;
        }

        *output = closest(palette, palette_components, *cin);
        output.alpha = 255;
    }
}

fn dither_threshold_apply(
    state: &I2PState,
    input: &[Color],
    output: &mut [Color],
    width: usize,
    height: usize,
    threshold: &[f32],
    dim: u8,
) {
    let amount = state.dither_options.dither_amount / 1000.0;

    for y in 0..height {
        for x in 0..width {
            let input = input[y * width + x];
            if input.alpha < state.dither_options.alpha_threshold {
                output[y * width + x] = Color::default();
                continue;
            }

            let r#mod = (1 << dim) - 1;
            let threshold_id = ((y & r#mod) << dim) + (x & r#mod);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let c = Color::new(
                0x0.max(0xff.min(
                    (f32::from(input.red) + 255.0 * amount * (threshold[threshold_id] - 0.5)) as u8,
                )),
                0x0.max(0xff.min(
                    (f32::from(input.green) + 255.0 * amount * (threshold[threshold_id] - 0.5))
                        as u8,
                )),
                0x0.max(0xff.min(
                    (f32::from(input.blue) + 255.0 * amount * (threshold[threshold_id] - 0.5))
                        as u8,
                )),
                255,
            );
            output[y * width + x] = c;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn dither_threshold(
    state: &I2PState,
    input: &[Color],
    output: &mut Vec<Color>,
    palette: &[Color],
    palette_components: &[Components],
    closest: impl Fn(&[Color], &[Components], Color) -> Color + Sync,
    width: usize,
    threshold: &[f32],
    dim: u8,
) {
    let amount = state.dither_options.dither_amount / 1000.0;

    input
        .par_iter()
        .enumerate()
        .map(|(i, input)| {
            let x = i % width;
            let y = i / width;
            if input.alpha < state.dither_options.alpha_threshold {
                return Color::default();
            }

            let r#mod = (1 << dim) - 1;
            let threshold_id = ((y & r#mod) << dim) + (x & r#mod);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let c = Color::new(
                0x0.max(0xff.min(
                    (f32::from(input.red) + 255.0 * amount * (threshold[threshold_id] - 0.5)) as u8,
                )),
                0x0.max(0xff.min(
                    (f32::from(input.green) + 255.0 * amount * (threshold[threshold_id] - 0.5))
                        as u8,
                )),
                0x0.max(0xff.min(
                    (f32::from(input.blue) + 255.0 * amount * (threshold[threshold_id] - 0.5))
                        as u8,
                )),
                255,
            );
            closest(palette, palette_components, c)
        })
        .collect_into_vec(output);
}
