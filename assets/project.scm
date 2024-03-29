(op! 'ramp "myRamp")
(op! 'noise "noise1")
(op! 'composite "composite")
(op! 'cuboid "cuboid1")
(op! 'standard-material "mat")
(op! 'window "window")

(param! (op "noise1") "Resolution" (list 100 100))
(param! (op "myRamp") "Resolution" (list 100 100))
(param! (op "composite") "Resolution" (list 100 100))
(param! (op "noise1") "Strength" (/ *time* 10))
(param! (op "myRamp") "Mode" 2)
(param! (op "window") "Texture" (op "mat"))
(param! (op "mat") "Texture" (op "composite"))