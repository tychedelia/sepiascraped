(op! 'ramp "myRamp")
(op! 'ramp "myRamp2")
(op! 'composite "composite")
(op! 'window "display")
(op! 'window "display2")

(param! (op "display") "Texture" (op "myRamp"))
(param! (op "display2") "Texture" (op "composite"))

(param! (op "myRamp") "Color A" (vector (rand 0.0 1.0) (rand 0.0 1.0) (rand 0.0 1.0) 1.0))
(param! (op "myRamp") "Color B" (vector (rand 0.0 1.0) (rand 0.0 1.0) (rand 0.0 1.0) 1.0))
(param! (op "myRamp") "Mode" 2)