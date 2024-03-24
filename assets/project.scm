(op! 'ramp "myRamp")
(op! 'noise "noise1")
(op! 'composite "composite")
(op! 'cuboid "cuboid1")
(op! 'window "window")

(param! (op "noise1") "Strength" (/ *time* 10))
(param! (op "myRamp") "Mode" 2)
(param! (op "window") "Texture" (op "cuboid1"))