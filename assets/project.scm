(op! 'ramp "myRamp")
(op! 'noise "noise1")
(op! 'composite "composite")
;(connect! (op "noise1") 0 (op "composite") 0)
;(connect! (op "myRamp") 0 (op "composite") 1)
;(op! 'cuboid "cuboid1")
;(op! 'grid "grid1")
;(op! 'mesh-noise "meshNoise")
;(op! 'standard-material "mat")
;;(op! 'camera "cam")
;;(op! 'light "light")
;(op! 'geom "geom")
;;(op! 'window "window")
;
;(param! (op "noise1") "Resolution" (list 100 100))
;(param! (op "myRamp") "Resolution" (list 100 100))
;(param! (op "composite") "Resolution" (list 100 100))
;(param! (op "noise1") "Strength" (/ *time* 10))
;(param! (op "myRamp") "Mode" 2)
;;(param! (op "meshNoise") "Strength" (rand 0.1 0.5))
;;(param! (op "window") "Texture" (op "mat"))
;(param! (op "mat") "Texture" (op "composite"))