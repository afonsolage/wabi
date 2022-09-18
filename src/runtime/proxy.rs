#![allow(unused)]
use std::{
    any::TypeId,
    borrow::{Borrow, Cow},
    cell::Ref,
};

use bevy::{
    ecs::schedule::IntoSystemDescriptor,
    prelude::{BVec3, Deref, DerefMut, IntoSystem, Mat4, Quat, Transform, Vec3, Vec4},
    render::render_resource::AsBindGroupShaderType,
};
use bevy_reflect::{Array, DynamicArray, FromReflect, GetField, Reflect, ReflectRef, TypePath};

#[derive(Reflect, Deref, DerefMut)]
pub struct TransformW(Transform);

impl FromReflect for TransformW {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        if let ReflectRef::Struct(s) = reflect.reflect_ref() {
            Some(TransformW(Transform {
                translation: *s.get_field("translation").unwrap(),
                rotation: *s.get_field("rotation").unwrap(),
                scale: *s.get_field("scale").unwrap(),
            }))
        } else {
            None
        }
    }
}

pub trait Proxy {
    fn new(code: u8, reflect: &dyn Reflect) -> Self;
    fn run(self) -> Box<dyn Reflect>;
}

pub enum TransformProxy {
    Identity,
    FromXYZ((f32, f32, f32)),
    FromMatrix(Mat4),
    FromTranslation(Vec3),
    FromRotation(Quat),
    FromScale(Vec3),
    LookingAt((TransformW, Vec3, Vec3)),
    WithTranslation((TransformW, Vec3)),
    WithRotation((TransformW, Quat)),
    WithScale((TransformW, Vec3)),
    ComputeMatrix(TransformW),
    ComputeAffine(TransformW),
    LocalX(TransformW),
    Left(TransformW),
    Right(TransformW),
    LocalY(TransformW),
    Up(TransformW),
    Down(TransformW),
    LocalZ(TransformW),
    Forward(TransformW),
    Back(TransformW),
    Rotate((TransformW, Quat)),
    RotateAxis((TransformW, Vec3, f32)),
    RotateX((TransformW, f32)),
    RotateY((TransformW, f32)),
    RotateZ((TransformW, f32)),
    RotateLocal((TransformW, Quat)),
    RotateLocalAxis((TransformW, Vec3, f32)),
    RotateLocalX((TransformW, f32)),
    RotateLocalY((TransformW, f32)),
    RotateLocalZ((TransformW, f32)),
    TranslateAround((TransformW, Vec3, Quat)),
    RotateAround((TransformW, Vec3, Quat)),
    LookAt((TransformW, Vec3, Vec3)),
    MulTransform((TransformW, TransformW)),
    MulVec3((TransformW, Vec3)),
    ApplyNonUniformScale((TransformW, Vec3)),
}

impl Proxy for TransformProxy {
    fn new(code: u8, reflect: &dyn Reflect) -> Self {
        match code {
            0 => TransformProxy::Identity,
            1 => TransformProxy::FromXYZ(FromReflect::from_reflect(reflect).unwrap()),
            2 => TransformProxy::FromMatrix(FromReflect::from_reflect(reflect).unwrap()),
            3 => TransformProxy::FromTranslation(FromReflect::from_reflect(reflect).unwrap()),
            4 => TransformProxy::FromRotation(FromReflect::from_reflect(reflect).unwrap()),
            5 => TransformProxy::FromScale(FromReflect::from_reflect(reflect).unwrap()),
            6 => TransformProxy::LookingAt(FromReflect::from_reflect(reflect).unwrap()),
            7 => TransformProxy::WithTranslation(FromReflect::from_reflect(reflect).unwrap()),
            8 => TransformProxy::WithRotation(FromReflect::from_reflect(reflect).unwrap()),
            9 => TransformProxy::WithScale(FromReflect::from_reflect(reflect).unwrap()),
            10 => TransformProxy::ComputeMatrix(FromReflect::from_reflect(reflect).unwrap()),
            11 => TransformProxy::ComputeAffine(FromReflect::from_reflect(reflect).unwrap()),
            12 => TransformProxy::LocalX(FromReflect::from_reflect(reflect).unwrap()),
            13 => TransformProxy::Left(FromReflect::from_reflect(reflect).unwrap()),
            14 => TransformProxy::Right(FromReflect::from_reflect(reflect).unwrap()),
            15 => TransformProxy::LocalY(FromReflect::from_reflect(reflect).unwrap()),
            16 => TransformProxy::Up(FromReflect::from_reflect(reflect).unwrap()),
            17 => TransformProxy::Down(FromReflect::from_reflect(reflect).unwrap()),
            18 => TransformProxy::LocalZ(FromReflect::from_reflect(reflect).unwrap()),
            19 => TransformProxy::Forward(FromReflect::from_reflect(reflect).unwrap()),
            20 => TransformProxy::Back(FromReflect::from_reflect(reflect).unwrap()),
            21 => TransformProxy::Rotate(FromReflect::from_reflect(reflect).unwrap()),
            22 => TransformProxy::RotateAxis(FromReflect::from_reflect(reflect).unwrap()),
            23 => TransformProxy::RotateX(FromReflect::from_reflect(reflect).unwrap()),
            24 => TransformProxy::RotateY(FromReflect::from_reflect(reflect).unwrap()),
            25 => TransformProxy::RotateZ(FromReflect::from_reflect(reflect).unwrap()),
            26 => TransformProxy::RotateLocal(FromReflect::from_reflect(reflect).unwrap()),
            27 => TransformProxy::RotateLocalAxis(FromReflect::from_reflect(reflect).unwrap()),
            28 => TransformProxy::RotateLocalX(FromReflect::from_reflect(reflect).unwrap()),
            29 => TransformProxy::RotateLocalY(FromReflect::from_reflect(reflect).unwrap()),
            30 => TransformProxy::RotateLocalZ(FromReflect::from_reflect(reflect).unwrap()),
            31 => TransformProxy::TranslateAround(FromReflect::from_reflect(reflect).unwrap()),
            32 => TransformProxy::RotateAround(FromReflect::from_reflect(reflect).unwrap()),
            33 => TransformProxy::LookAt(FromReflect::from_reflect(reflect).unwrap()),
            34 => TransformProxy::MulTransform(FromReflect::from_reflect(reflect).unwrap()),
            35 => TransformProxy::MulVec3(FromReflect::from_reflect(reflect).unwrap()),
            36 => TransformProxy::ApplyNonUniformScale(FromReflect::from_reflect(reflect).unwrap()),
            _ => unreachable!(),
        }
    }

    fn run(self) -> Box<dyn Reflect> {
        match self {
            TransformProxy::Identity => Box::new(Transform::IDENTITY),
            TransformProxy::FromXYZ((x, y, z)) => Box::new(Transform::from_xyz(x, y, z)),
            TransformProxy::FromMatrix(matrix) => Box::new(Transform::from_matrix(matrix)),
            TransformProxy::FromTranslation(translation) => {
                Box::new(Transform::from_translation(translation))
            }
            TransformProxy::FromRotation(rotation) => Box::new(Transform::from_rotation(rotation)),
            TransformProxy::FromScale(scale) => Box::new(Transform::from_scale(scale)),
            TransformProxy::LookingAt((mut itself, target, up)) => {
                Box::new(Transform::looking_at(*itself, target, up))
            }
            TransformProxy::WithTranslation((mut itself, translation)) => {
                Box::new(Transform::with_translation(*itself, translation))
            }
            TransformProxy::WithRotation((mut itself, rotation)) => {
                Box::new(Transform::with_rotation(*itself, rotation))
            }
            TransformProxy::WithScale((mut itself, scale)) => {
                Box::new(Transform::with_scale(*itself, scale))
            }
            TransformProxy::ComputeMatrix(itself) => Box::new(Transform::compute_matrix(&itself)),
            TransformProxy::ComputeAffine(itself) => Box::new(Transform::compute_affine(&itself)),
            TransformProxy::LocalX(itself) => Box::new(Transform::local_x(&itself)),
            TransformProxy::Left(itself) => Box::new(Transform::left(&itself)),
            TransformProxy::Right(itself) => Box::new(Transform::right(&itself)),
            TransformProxy::LocalY(itself) => Box::new(Transform::local_y(&itself)),
            TransformProxy::Up(itself) => Box::new(Transform::up(&itself)),
            TransformProxy::Down(itself) => Box::new(Transform::down(&itself)),
            TransformProxy::LocalZ(itself) => Box::new(Transform::local_z(&itself)),
            TransformProxy::Forward(itself) => Box::new(Transform::forward(&itself)),
            TransformProxy::Back(itself) => Box::new(Transform::back(&itself)),
            TransformProxy::Rotate((mut itself, rotation)) => {
                Transform::rotate(&mut itself, rotation);
                Box::new(*itself)
            }
            TransformProxy::RotateAxis((mut itself, axis, angle)) => {
                Transform::rotate_axis(&mut itself, axis, angle);
                Box::new(*itself)
            }
            TransformProxy::RotateX((mut itself, angle)) => {
                Transform::rotate_x(&mut itself, angle);
                Box::new(*itself)
            }
            TransformProxy::RotateY((mut itself, angle)) => {
                Transform::rotate_y(&mut itself, angle);
                Box::new(*itself)
            }
            TransformProxy::RotateZ((mut itself, angle)) => {
                Transform::rotate_z(&mut itself, angle);
                Box::new(*itself)
            }
            TransformProxy::RotateLocal((mut itself, rotation)) => {
                Transform::rotate_local(&mut itself, rotation);
                Box::new(*itself)
            }
            TransformProxy::RotateLocalAxis((mut itself, axis, angle)) => {
                Transform::rotate_local_axis(&mut itself, axis, angle);
                Box::new(*itself)
            }
            TransformProxy::RotateLocalX((mut itself, angle)) => {
                Transform::rotate_local_x(&mut itself, angle);
                Box::new(*itself)
            }
            TransformProxy::RotateLocalY((mut itself, angle)) => {
                Transform::rotate_local_y(&mut itself, angle);
                Box::new(*itself)
            }
            TransformProxy::RotateLocalZ((mut itself, angle)) => {
                Transform::rotate_local_z(&mut itself, angle);
                Box::new(*itself)
            }
            TransformProxy::TranslateAround((mut itself, point, rotation)) => {
                Transform::translate_around(&mut itself, point, rotation);
                Box::new(*itself)
            }
            TransformProxy::RotateAround((mut itself, point, rotation)) => {
                Transform::rotate_around(&mut itself, point, rotation);
                Box::new(*itself)
            }
            TransformProxy::LookAt((mut itself, target, up)) => {
                Transform::look_at(&mut itself, target, up);
                Box::new(*itself)
            }
            TransformProxy::MulTransform((itself, transform)) => {
                Box::new(Transform::mul_transform(&itself, *transform))
            }
            TransformProxy::MulVec3((itself, value)) => {
                Box::new(Transform::mul_vec3(&itself, value))
            }
            TransformProxy::ApplyNonUniformScale((mut itself, scale_factor)) => {
                Transform::apply_non_uniform_scale(&mut itself, scale_factor);
                Box::new(*itself)
            }
        }
    }
}

pub enum Vec3Proxy {
    Zero,
    New((f32, f32, f32)),
    Splat(f32),
    Select((BVec3, Vec3, Vec3)),
    FromArray([f32; 3]),
    ToArray(Vec3),
    FromSlice(Vec<f32>),
    WriteToSlice((Vec3, Vec<f32>)),
    Extend((Vec3, f32)),
    Truncate(Vec3),
    Dot((Vec3, Vec3)),
    Cross((Vec3, Vec3)),
    Min((Vec3, Vec3)),
    Max((Vec3, Vec3)),
    Clamp((Vec3, Vec3, Vec3)),
}

impl Proxy for Vec3Proxy {
    fn new(code: u8, reflect: &dyn Reflect) -> Self {
        match code {
            0 => Vec3Proxy::Zero,
            1 => Vec3Proxy::New(FromReflect::from_reflect(reflect).unwrap()),
            2 => Vec3Proxy::Splat(FromReflect::from_reflect(reflect).unwrap()),
            3 => Vec3Proxy::Select(FromReflect::from_reflect(reflect).unwrap()),
            4 => Vec3Proxy::FromArray(FromReflect::from_reflect(reflect).unwrap()),
            5 => Vec3Proxy::ToArray(FromReflect::from_reflect(reflect).unwrap()),
            6 => Vec3Proxy::FromSlice(FromReflect::from_reflect(reflect).unwrap()),
            7 => Vec3Proxy::WriteToSlice(FromReflect::from_reflect(reflect).unwrap()),
            8 => Vec3Proxy::Extend(FromReflect::from_reflect(reflect).unwrap()),
            9 => Vec3Proxy::Truncate(FromReflect::from_reflect(reflect).unwrap()),
            10 => Vec3Proxy::Dot(FromReflect::from_reflect(reflect).unwrap()),
            12 => Vec3Proxy::Cross(FromReflect::from_reflect(reflect).unwrap()),
            13 => Vec3Proxy::Min(FromReflect::from_reflect(reflect).unwrap()),
            14 => Vec3Proxy::Max(FromReflect::from_reflect(reflect).unwrap()),
            15 => Vec3Proxy::Clamp(FromReflect::from_reflect(reflect).unwrap()),
            _ => unreachable!(),
        }
    }

    fn run(self) -> Box<dyn Reflect> {
        match self {
            Vec3Proxy::Zero => Box::new(Vec3::ZERO),
            Vec3Proxy::New((x, y, z)) => Box::new(Vec3::new(x, y, z)),
            Vec3Proxy::Splat(v) => Box::new(Vec3::splat(v)),
            Vec3Proxy::Select((mask, if_true, if_false)) => {
                Box::new(Vec3::select(mask, if_true, if_false))
            }
            Vec3Proxy::FromArray(a) => Box::new(Vec3::from_array(a)),
            Vec3Proxy::ToArray(itself) => Box::new(Vec3::to_array(&itself)),
            Vec3Proxy::FromSlice(slice) => Box::new(Vec3::from_slice(&slice)),
            Vec3Proxy::WriteToSlice((itself, mut slice)) => {
                Vec3::write_to_slice(itself, &mut slice);
                Box::new(slice)
            }
            Vec3Proxy::Extend((itself, w)) => Box::new(Vec3::extend(itself, w)),
            Vec3Proxy::Truncate(itself) => Box::new(Vec3::truncate(itself)),
            Vec3Proxy::Dot((itself, rhs)) => Box::new(Vec3::dot(itself, rhs)),
            Vec3Proxy::Cross((itself, rhs)) => Box::new(Vec3::cross(itself, rhs)),
            Vec3Proxy::Min((itself, rhs)) => Box::new(Vec3::min(itself, rhs)),
            Vec3Proxy::Max((itself, rhs)) => Box::new(Vec3::max(itself, rhs)),
            Vec3Proxy::Clamp((itself, min, max)) => Box::new(Vec3::clamp(itself, min, max)),
        }
    }
}
