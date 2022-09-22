#![allow(unused)]
use std::{
    any::TypeId,
    borrow::{Borrow, Cow},
    cell::Ref,
};

use bevy::{
    ecs::schedule::IntoSystemDescriptor,
    prelude::{error, BVec3, Deref, DerefMut, IntoSystem, Mat4, Quat, Transform, Vec3, Vec4},
    render::render_resource::AsBindGroupShaderType,
};
use bevy_reflect::{
    Array, DynamicArray, FromReflect, GetField, Reflect, ReflectRef, TypePath, Typed,
};
use wabi_runtime_api::mod_api::proxy::TransformProxy;

pub(crate) fn call(data: Box<dyn Reflect>) -> Option<Box<dyn Reflect>> {
    let result = match data.type_path() {
        p if p == <TransformProxy as TypePath>::type_path() => {
            TransformProxy::from_reflect(&*data).unwrap().run()
        }
        _ => {
            error!("Unknown rpc data: {}", data.get_type_info().type_path());
            return None;
        }
    };

    if result.is::<()>() {
        None
    } else {
        Some(result)
    }
}

trait Proxy {
    fn run(self) -> Box<dyn Reflect>;
}

impl Proxy for TransformProxy {
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
                Box::new(Transform::looking_at(itself, target, up))
            }
            TransformProxy::WithTranslation((mut itself, translation)) => {
                Box::new(Transform::with_translation(itself, translation))
            }
            TransformProxy::WithRotation((mut itself, rotation)) => {
                Box::new(Transform::with_rotation(itself, rotation))
            }
            TransformProxy::WithScale((mut itself, scale)) => {
                Box::new(Transform::with_scale(itself, scale))
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
                Box::new(itself)
            }
            TransformProxy::RotateAxis((mut itself, axis, angle)) => {
                Transform::rotate_axis(&mut itself, axis, angle);
                Box::new(itself)
            }
            TransformProxy::RotateX((mut itself, angle)) => {
                Transform::rotate_x(&mut itself, angle);
                Box::new(itself)
            }
            TransformProxy::RotateY((mut itself, angle)) => {
                Transform::rotate_y(&mut itself, angle);
                Box::new(itself)
            }
            TransformProxy::RotateZ((mut itself, angle)) => {
                Transform::rotate_z(&mut itself, angle);
                Box::new(itself)
            }
            TransformProxy::RotateLocal((mut itself, rotation)) => {
                Transform::rotate_local(&mut itself, rotation);
                Box::new(itself)
            }
            TransformProxy::RotateLocalAxis((mut itself, axis, angle)) => {
                Transform::rotate_local_axis(&mut itself, axis, angle);
                Box::new(itself)
            }
            TransformProxy::RotateLocalX((mut itself, angle)) => {
                Transform::rotate_local_x(&mut itself, angle);
                Box::new(itself)
            }
            TransformProxy::RotateLocalY((mut itself, angle)) => {
                Transform::rotate_local_y(&mut itself, angle);
                Box::new(itself)
            }
            TransformProxy::RotateLocalZ((mut itself, angle)) => {
                Transform::rotate_local_z(&mut itself, angle);
                Box::new(itself)
            }
            TransformProxy::TranslateAround((mut itself, point, rotation)) => {
                Transform::translate_around(&mut itself, point, rotation);
                Box::new(itself)
            }
            TransformProxy::RotateAround((mut itself, point, rotation)) => {
                Transform::rotate_around(&mut itself, point, rotation);
                Box::new(itself)
            }
            TransformProxy::LookAt((mut itself, target, up)) => {
                Transform::look_at(&mut itself, target, up);
                Box::new(itself)
            }
            TransformProxy::MulTransform((itself, transform)) => {
                Box::new(Transform::mul_transform(&itself, transform))
            }
            TransformProxy::MulVec3((itself, value)) => {
                Box::new(Transform::mul_vec3(&itself, value))
            }
            TransformProxy::ApplyNonUniformScale((mut itself, scale_factor)) => {
                Transform::apply_non_uniform_scale(&mut itself, scale_factor);
                Box::new(itself)
            }
        }
    }
}

enum Vec3Proxy {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call() {
        let transform = Transform::IDENTITY;
        let result = super::call(Box::new(TransformProxy::Identity)).unwrap();
        assert_eq!(transform, *result.downcast().unwrap());

        let transform = Transform::from_xyz(1.0, 2.0, 3.0);
        let result =
            super::call(Box::new(TransformProxy::FromXYZ((1.0f32, 2.0f32, 3.0f32)))).unwrap();
        assert_eq!(transform, *result.downcast().unwrap());

        let transform = Transform::from_xyz(5.0, 6.7, 8.0)
            .with_rotation(Quat::from_euler(
                bevy::prelude::EulerRot::XYZ,
                9.0,
                10.0,
                11.0,
            ))
            .with_scale(Vec3::splat(12.0));

        let vec = Vec3::new(1.0, 2.0, 3.0);

        let result = super::call(Box::new(TransformProxy::MulVec3((transform, vec)))).unwrap();
        assert_eq!(transform.mul_vec3(vec), *result.downcast().unwrap());
    }
}
