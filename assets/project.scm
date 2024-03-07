(op! 'ramp "myRamp")
(op! 'ramp "myRamp2")
(op! 'composite "composite")
(op! 'window "display")

(param! (op "display") "Texture" (op "composite"))

(param! (op "myRamp") "Color A" (vector (rand 0.0 1.0) (rand 0.0 1.0) (rand 0.0 1.0) 1.0))
(param! (op "myRamp") "Color B" (vector (rand 0.0 1.0) (rand 0.0 1.0) (rand 0.0 1.0) 1.0))
(param! (op "myRamp") "Mode" 2)