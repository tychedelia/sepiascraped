struct DisplaceSettings {
    uScale: vec2<f32>;
    uXForm: array<mat4x4<f32>, 4>;
    uPower: f32;
    uAmp: f32;
    uOffset: f32;
    uGain: f32;
    uLacunarity: f32;
    uMult: vec2<f32>;
};

@group(1) @binding(0)
var<uniform> displace_settings: DisplaceSettings;
@group(0) @binding(1)
var input_sampler: sampler;

@group(0) @binding(0)
var sTDPermTexture: texture_2d<f32>;

@location(0)
var<in> vTexCoord: array<vec3<f32>, 4>;

@location(0)
var<out> fragColor: vec4<f32>;

fn main() {
    var t: array<vec3<f32>, 4> = vTexCoord;

    var amp: f32 = defaultUniforms.uAmp;
    var lacunarity: f32 = defaultUniforms.uLacunarity;
    var gain: f32 = defaultUniforms.uGain;

    for (var i = 0; i < 4; i++) {
        t[i] = vTexCoord[i];
    }

    var noiseValue: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    for (var j = 0; j < 3; j++) {
        var currentPoint = t[0];
        var floorPoint = floor(currentPoint + vec3<f32>((currentPoint.x + currentPoint.y + currentPoint.z) * 0.3333));
        var adjustedPoint = currentPoint - (floorPoint - vec3<f32>((floorPoint.x + floorPoint.y + floorPoint.z) * 0.1667));
        var texCoord = (floorPoint * 0.0039) + vec3<f32>(0.0020, 0.0020, 0.0020);

        var gradSelection = textureSample(sTDPermTexture, sTDSimplexTexture, vec2<f32>((adjustedPoint.x > adjustedPoint.y ? 0.5078 : 0.0078) + (adjustedPoint.x > adjustedPoint.z ? 0.2500 : 0.0) + (adjustedPoint.y > adjustedPoint.z ? 0.1250 : 0.0)));
        var step1 = step(vec3<f32>(0.375, 0.375, 0.375), gradSelection.xyz);
        var step2 = step(vec3<f32>(0.125, 0.125, 0.125), gradSelection.xyz);

        var weight = 0.6000 - dot(adjustedPoint, adjustedPoint);
        var contribution: f32 = 0.0;
        if (weight > 0.0) {
            let weightSquared = weight * weight;
            contribution = (weightSquared * weightSquared) * dot((textureSample(sTDPermTexture, sTDSimplexTexture, vec2<f32>(texCoord.xy)).xyz * 4.0) - vec3<f32>(1.0, 1.0, 1.0), adjustedPoint);
        }

        noiseValue = vec4<f32>(fma(32.0 * contribution, amp, noiseValue.x), noiseValue.yzw);
        t[0] *= lacunarity;
        amp *= gain;
    }

    let powAbsNoise = pow(abs(noiseValue), vec4<f32>(defaultUniforms.uPower));
    let finalColor = fma(powAbsNoise, sign(noiseValue), vec4<f32>(defaultUniforms.uOffset)) * vec4<f32>(defaultUniforms.uMult.y, defaultUniforms.uMult.y, defaultUniforms.uMult.y, 1.0);
    fragColor = finalColor;
}