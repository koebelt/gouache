# Match `01_tinyglade_town` iter 09

Gated regions pass: **NO**

## whole_frame FAIL

| metric | ref | ours | distance | limit | |
|---|---|---|---|---|---|
| palette_de | 0.000 | 0.000 | 5.340 | <= 6 | ok |
| l_hist_emd | 0.646 | 0.678 | 0.096 | <= 0.035 | **x** |
| chroma_hist_l1 | 0.084 | 0.098 | 0.661 | <= 0.5 | **x** |
| hue_hist_l1 | 0.000 | 0.000 | 0.819 | <= 0.6 | **x** |
| shadow_light_ratio | 0.452 | 0.594 | 0.142 | <= 0.08 | **x** |
| warm_cool_sep | 0.066 | 0.010 | 0.056 | <= 0.025 | **x** |
| local_contrast | 0.045 | 0.009 | 0.203 | ratio in [0.7, 1.45] | **x** |
| edge_density | 0.535 | 0.072 | 0.134 | ratio in [0.6, 1.7] | **x** |
| spectrum_rms | 0.000 | 0.000 | 0.256 | <= 0.45 | ok |

## backdrop_haze FAIL (gated)

| metric | ref | ours | distance | limit | |
|---|---|---|---|---|---|
| palette_de | 0.000 | 0.000 | 10.973 | <= 6 | **x** |
| l_hist_emd | 0.788 | 0.680 | 0.112 | <= 0.035 | **x** |
| chroma_hist_l1 | 0.091 | 0.046 | 1.715 | <= 0.5 | **x** |
| hue_hist_l1 | 0.000 | 0.000 | 1.266 | <= 0.6 | **x** |
| shadow_light_ratio | 0.723 | 0.755 | 0.032 | <= 0.08 | ok |
| warm_cool_sep | 0.037 | 0.040 | 0.003 | <= 0.025 | ok |
| local_contrast | 0.031 | 0.009 | 0.293 | ratio in [0.7, 1.45] | x (info) |
| edge_density | 0.444 | 0.042 | 0.094 | ratio in [0.6, 1.7] | x (info) |
| spectrum_rms | 0.000 | 0.000 | 1.642 | <= 0.45 | x (info) |

## shadow_edge FAIL (gated)

| metric | ref | ours | distance | limit | |
|---|---|---|---|---|---|
| palette_de | 0.000 | 0.000 | 4.888 | <= 6 | ok |
| l_hist_emd | 0.609 | 0.482 | 0.130 | <= 0.035 | **x** |
| chroma_hist_l1 | 0.087 | 0.070 | 0.921 | <= 0.5 | **x** |
| hue_hist_l1 | 0.000 | 0.000 | 0.858 | <= 0.6 | **x** |
| shadow_light_ratio | 0.473 | 0.564 | 0.091 | <= 0.08 | **x** |
| warm_cool_sep | 0.063 | 0.038 | 0.025 | <= 0.025 | **x** |
| local_contrast | 0.041 | 0.014 | 0.332 | ratio in [0.7, 1.45] | x (info) |
| edge_density | 0.527 | 0.142 | 0.270 | ratio in [0.6, 1.7] | x (info) |
| spectrum_rms | 0.000 | 0.000 | 1.756 | <= 0.45 | x (info) |

## grass_with_bounce_light FAIL (gated)

| metric | ref | ours | distance | limit | |
|---|---|---|---|---|---|
| palette_de | 0.000 | 0.000 | 6.418 | <= 6 | **x** |
| l_hist_emd | 0.777 | 0.702 | 0.078 | <= 0.035 | **x** |
| chroma_hist_l1 | 0.121 | 0.121 | 0.273 | <= 0.5 | ok |
| hue_hist_l1 | 0.000 | 0.000 | 0.553 | <= 0.6 | ok |
| shadow_light_ratio | 0.729 | 0.835 | 0.106 | <= 0.08 | **x** |
| warm_cool_sep | -0.010 | 0.009 | 0.018 | <= 0.025 | ok |
| local_contrast | 0.049 | 0.010 | 0.202 | ratio in [0.7, 1.45] | x (info) |
| edge_density | 0.616 | 0.085 | 0.138 | ratio in [0.6, 1.7] | x (info) |
| spectrum_rms | 0.000 | 0.000 | 1.170 | <= 0.45 | x (info) |

## tree_canopy FAIL

| metric | ref | ours | distance | limit | |
|---|---|---|---|---|---|
| palette_de | 0.000 | 0.000 | 7.574 | <= 6 | **x** |
| l_hist_emd | 0.664 | 0.704 | 0.113 | <= 0.035 | **x** |
| chroma_hist_l1 | 0.082 | 0.060 | 1.358 | <= 0.5 | **x** |
| hue_hist_l1 | 0.000 | 0.000 | 0.454 | <= 0.6 | ok |
| shadow_light_ratio | 0.473 | 0.751 | 0.278 | <= 0.08 | **x** |
| warm_cool_sep | 0.070 | 0.009 | 0.062 | <= 0.025 | **x** |
| local_contrast | 0.042 | 0.006 | 0.143 | ratio in [0.7, 1.45] | **x** |
| edge_density | 0.499 | 0.042 | 0.084 | ratio in [0.6, 1.7] | **x** |
| spectrum_rms | 0.000 | 0.000 | 0.857 | <= 0.45 | **x** |

## stone_wall_base FAIL

| metric | ref | ours | distance | limit | |
|---|---|---|---|---|---|
| palette_de | 0.000 | 0.000 | 6.190 | <= 6 | **x** |
| l_hist_emd | 0.671 | 0.665 | 0.073 | <= 0.035 | **x** |
| chroma_hist_l1 | 0.079 | 0.106 | 1.062 | <= 0.5 | **x** |
| hue_hist_l1 | 0.000 | 0.000 | 0.980 | <= 0.6 | **x** |
| shadow_light_ratio | 0.532 | 0.576 | 0.043 | <= 0.08 | ok |
| warm_cool_sep | 0.011 | -0.025 | 0.037 | <= 0.025 | **x** |
| local_contrast | 0.039 | 0.019 | 0.499 | ratio in [0.7, 1.45] | **x** |
| edge_density | 0.466 | 0.160 | 0.344 | ratio in [0.6, 1.7] | **x** |
| spectrum_rms | 0.000 | 0.000 | 0.473 | <= 0.45 | **x** |

## roof_teal_slates FAIL

| metric | ref | ours | distance | limit | |
|---|---|---|---|---|---|
| palette_de | 0.000 | 0.000 | 6.947 | <= 6 | **x** |
| l_hist_emd | 0.656 | 0.662 | 0.084 | <= 0.035 | **x** |
| chroma_hist_l1 | 0.061 | 0.085 | 1.181 | <= 0.5 | **x** |
| hue_hist_l1 | 0.000 | 0.000 | 1.027 | <= 0.6 | **x** |
| shadow_light_ratio | 0.495 | 0.632 | 0.137 | <= 0.08 | **x** |
| warm_cool_sep | 0.052 | 0.025 | 0.027 | <= 0.025 | **x** |
| local_contrast | 0.072 | 0.012 | 0.166 | ratio in [0.7, 1.45] | **x** |
| edge_density | 0.658 | 0.088 | 0.134 | ratio in [0.6, 1.7] | **x** |
| spectrum_rms | 0.000 | 0.000 | 1.095 | <= 0.45 | **x** |

