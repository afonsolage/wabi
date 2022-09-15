macro_rules! impl_type {
    ($ty:ident) => {
        #[derive(Debug)]
        pub struct $ty(Box<dyn Reflect>);

        impl bevy_reflect::Reflect for $ty {
            fn type_path(&self) -> &str {
                self.0.type_path()
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

        impl bevy_reflect::TypePath for $ty {
            #[inline]
            fn type_path() -> &'static str {
                concat!(concat!(module_path!(), "::"), stringify!($ty))
            }
            #[inline]
            fn short_type_name_base() -> &'static str {
                const IDENT_POS: usize = module_path!().len() + 2;
                const GENERIC_POS: usize = IDENT_POS + stringify!($ty).len();
                &<Self as bevy_reflect::TypePath>::type_path()[IDENT_POS..GENERIC_POS]
            }
            #[inline]
            fn short_type_name() -> &'static str {
                const IDENT_POS: usize = module_path!().len() + 2;
                &<Self as bevy_reflect::TypePath>::type_path()[IDENT_POS..]
            }
            #[inline]
            fn module_path() -> &'static str {
                &<Self as bevy_reflect::TypePath>::type_path()[..module_path!().len()]
            }
            #[inline]
            fn crate_name() -> &'static str {
                &<Self as bevy_reflect::TypePath>::type_path()
                    [..bevy_reflect::utility::crate_name_len(module_path!())]
            }
        }
    };
}

pub(crate) use impl_type;
