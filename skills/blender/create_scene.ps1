param([string]$jsonArgs = "{}")

$ErrorActionPreference = "Stop"
$parsed = ConvertFrom-Json $jsonArgs -ErrorAction SilentlyContinue
if (-not $parsed) { $parsed = [pscustomobject]@{} }

# 1. Locate Blender executable
$blenderPath = "C:\Program Files\Blender Foundation\Blender 4.4\blender.exe"
if (-not (Test-Path $blenderPath)) {
    $found = Get-Command "blender.exe" -ErrorAction SilentlyContinue
    if ($found) { $blenderPath = $found.Source }
}
if (-not (Test-Path $blenderPath)) {
    $candidates = Get-ChildItem "C:\Program Files\Blender Foundation" -Filter "blender.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($candidates) { $blenderPath = $candidates.FullName }
}
if (-not (Test-Path $blenderPath)) {
    Write-Output "Error: Blender executable not found. Please verify Blender is installed at 'C:\Program Files\Blender Foundation'."
    exit 1
}

# 2. Determine target save location
$scenesDir = Join-Path $HOME "Documents\BlenderScenes"
if (-not (Test-Path $scenesDir)) { New-Item -ItemType Directory -Path $scenesDir -Force | Out-Null }
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$outBlend = $parsed.output_blend_path
if ([string]::IsNullOrWhiteSpace($outBlend)) {
    $outBlend = Join-Path $scenesDir "Scene_$timestamp.blend"
}

$desc = if ($parsed.description) { $parsed.description } else { "Modern minimalist 3D scene" }
$sceneType = if ($parsed.scene_type) { $parsed.scene_type } else { "abstract_art" }
$openGui = if ($null -ne $parsed.open_gui) { [bool]$parsed.open_gui } else { $true }

# 3. Generate procedural bpy Python scene construction script
$tempPy = [System.IO.Path]::GetTempFileName() + ".py"
$pyContent = @"
import bpy
import math

# Clear existing scene
bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.render.engine = 'CYCLES' if 'CYCLES' in [e.identifier for e in bpy.types.RenderEngine.bl_rna.properties['engine'].enum_items] else 'BLENDER_EEVEE_NEXT'

# Create Camera
cam_data = bpy.data.cameras.new(name="MainCamera")
cam_obj = bpy.data.objects.new(name="MainCamera", object_data=cam_data)
bpy.context.collection.objects.link(cam_obj)
scene.camera = cam_obj
cam_obj.location = (7.35889, -6.92579, 4.95831)
cam_obj.rotation_euler = (math.radians(63.6), 0.0, math.radians(46.7))

# 3-Point Lighting Setup
# Key Light
key_data = bpy.data.lights.new(name="KeyLight", type='AREA')
key_data.energy = 500
key_data.size = 2.0
key_obj = bpy.data.objects.new(name="KeyLight", object_data=key_data)
bpy.context.collection.objects.link(key_obj)
key_obj.location = (4.0, -4.0, 5.0)

# Fill Light (Cool tint)
fill_data = bpy.data.lights.new(name="FillLight", type='AREA')
fill_data.energy = 200
fill_data.color = (0.7, 0.85, 1.0)
fill_obj = bpy.data.objects.new(name="FillLight", object_data=fill_data)
bpy.context.collection.objects.link(fill_obj)
fill_obj.location = (-4.0, -3.0, 3.0)

# Rim Light (Accent red)
rim_data = bpy.data.lights.new(name="RimLight", type='POINT')
rim_data.energy = 350
rim_data.color = (1.0, 0.2, 0.2)
rim_obj = bpy.data.objects.new(name="RimLight", object_data=rim_data)
bpy.context.collection.objects.link(rim_obj)
rim_obj.location = (-2.0, 4.0, 4.0)

# Base Ground / Studio Backdrop
bpy.ops.mesh.primitive_plane_add(size=30, location=(0, 0, 0))
floor = bpy.context.active_object
floor.name = "Floor"
floor_mat = bpy.data.materials.new(name="FloorMat")
floor_mat.use_nodes = True
bsdf = floor_mat.node_tree.nodes.get("Principled BSDF")
if bsdf:
    bsdf.inputs['Base Color'].default_value = (0.05, 0.05, 0.07, 1.0)
    bsdf.inputs['Roughness'].default_value = 0.2
floor.data.materials.append(floor_mat)

# Scene-specific geometry
scene_type = "$sceneType".lower()

if "room" in scene_type:
    # Build stylized room geometry
    bpy.ops.mesh.primitive_cube_add(size=2, location=(0, 0, 1))
    desk = bpy.context.active_object
    desk.scale = (1.8, 1.0, 0.4)
    mat = bpy.data.materials.new(name="WoodDesk")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    if bsdf: bsdf.inputs['Base Color'].default_value = (0.35, 0.18, 0.08, 1.0)
    desk.data.materials.append(mat)
else:
    # Abstract artistic composition (Hero sphere + orbiting toruses + pedestal)
    bpy.ops.mesh.primitive_cylinder_add(radius=1.5, depth=0.4, location=(0, 0, 0.2))
    pedestal = bpy.context.active_object
    ped_mat = bpy.data.materials.new(name="PedestalMat")
    ped_mat.use_nodes = True
    bsdf = ped_mat.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs['Base Color'].default_value = (0.02, 0.02, 0.03, 1.0)
        bsdf.inputs['Metallic'].default_value = 0.8
        bsdf.inputs['Roughness'].default_value = 0.1
    pedestal.data.materials.append(ped_mat)

    # Hero Element (Sphere)
    bpy.ops.mesh.primitive_uv_sphere_add(radius=1.0, location=(0, 0, 1.8))
    hero = bpy.context.active_object
    hero.name = "HeroSphere"
    hero_mat = bpy.data.materials.new(name="HeroMat")
    hero_mat.use_nodes = True
    bsdf = hero_mat.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs['Base Color'].default_value = (0.9, 0.05, 0.05, 1.0)
        bsdf.inputs['Metallic'].default_value = 0.95
        bsdf.inputs['Roughness'].default_value = 0.08
    hero.data.materials.append(hero_mat)

    # Orbiting Torus Ring
    bpy.ops.mesh.primitive_torus_add(major_radius=1.7, minor_radius=0.08, location=(0, 0, 1.8))
    ring = bpy.context.active_object
    ring.rotation_euler = (math.radians(35), math.radians(20), 0)
    ring_mat = bpy.data.materials.new(name="NeonRing")
    ring_mat.use_nodes = True
    bsdf = ring_mat.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs['Emission Color'].default_value = (1.0, 0.15, 0.15, 1.0)
        if 'Emission Strength' in bsdf.inputs:
            bsdf.inputs['Emission Strength'].default_value = 5.0
    ring.data.materials.append(ring_mat)

# Save to target .blend file
target_path = r"$outBlend"
bpy.ops.wm.save_as_mainfile(filepath=target_path)
print(f"SUCCESS: Scene saved to {target_path}")
"@

Set-Content -Path $tempPy -Value $pyContent -Encoding UTF8

# 4. Execute Blender
if ($openGui) {
    # Run script in background to build scene, then launch Blender GUI with saved scene
    & $blenderPath --background --python $tempPy | Out-Null
    Start-Process $blenderPath -ArgumentList "`"$outBlend`""
    Write-Output "✓ 3D Scene created successfully! Saved to: $outBlend. Blender GUI launched."
} else {
    $out = & $blenderPath --background --python $tempPy
    Write-Output "✓ 3D Scene created headlessly: $outBlend"
}

Remove-Item $tempPy -ErrorAction SilentlyContinue

