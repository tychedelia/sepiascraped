(op! 'ramp "myRamp")
(op! 'noise "noise1")
(op! 'composite "composite")

(connect! (op "myRamp") (op "composite"))
(connect! (op "noise1") (op "composite"))

(param! (op "noise1") "Strength" (/ *time* 10))
(param! (op "display") "Texture" (op "myRamp"))
(param! (op "display2") "Texture" (op "noise1"))
(param! (op "myRamp") "Mode" 1)

(op! 'window "display")
(op! 'window "display2")