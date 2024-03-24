(op! 'ramp "myRamp")
(op! 'noise "noise1")
(op! 'composite "composite")

(param! (op "noise1") "Strength" (/ *time* 10))
(param! (op "myRamp") "Mode" 2)