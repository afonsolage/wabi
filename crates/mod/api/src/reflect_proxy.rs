macro_rules! impl_type {
    ($ty:ident) => {
        #[derive(Debug)]
        pub struct $ty(Box<dyn Reflect>);

        impl bevy_reflect::Reflect for $ty {
            fn type_name(&self) -> &str {
                self.0.type_name()
            }

            fn get_type_info(&self) -> &'static bevy_reflect::TypeInfo {
                self.0.get_type_info()
            }

            fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
                self.0.into_any()
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self.0.as_any()
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self.0.as_any_mut()
            }

            fn as_reflect(&self) -> &dyn Reflect {
                self.0.as_reflect()
            }

            fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
                self.0.as_reflect_mut()
            }

            fn apply(&mut self, value: &dyn Reflect) {
                self.0.apply(value)
            }

            fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
                self.0.set(value)
            }

            fn reflect_ref(&self) -> bevy_reflect::ReflectRef {
                self.0.reflect_ref()
            }

            fn reflect_mut(&mut self) -> bevy_reflect::ReflectMut {
                self.0.reflect_mut()
            }

            fn clone_value(&self) -> Box<dyn Reflect> {
                self.0.clone_value()
            }
        }

        impl FromReflect for $ty {
            fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                Some(Self(reflect.clone_value()))
            }
        }

        impl From<&dyn Reflect> for $ty {
            fn from(reflect: &dyn Reflect) -> Self {
                Self(reflect.clone_value())
            }
        }

        impl From<Box<dyn Reflect>> for $ty {
            fn from(reflect: Box<dyn Reflect>) -> Self {
                Self(reflect)
            }
        }
    };
}

pub(crate) use impl_type;
