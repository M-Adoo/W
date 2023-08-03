#[macro_export]
macro_rules! impl_all_event {
  ($name: ident, $($on_doc: literal, $event_ty: ident),+) => {
    paste::paste! {
      #[doc="All `" $name:snake "` related events"]
      pub enum [<All $name>] {
        $(
          #[doc = $on_doc]
          $event_ty([<$name Event>]),
        )+
      }

      impl std::ops::Deref for [<All $name>] {
        type Target = [<$name Event>];
        fn deref(&self) -> &Self::Target {
          match self {
            $([<All $name>]::$event_ty(e)) |+ => e
          }
        }
      }

      impl std::ops::DerefMut for [<All $name>] {
        fn deref_mut(&mut self) -> &mut Self::Target {
          match self {
            $([<All $name>]::$event_ty(e)) |+ => e
          }
        }
      }

      impl [<All $name>] {
        pub fn into_inner(self) -> [<$name Event>] {
          match self {
            $([<All $name>]::$event_ty(e)) |+ => e
          }
        }
      }
    }
  };
}

#[macro_export]
macro_rules! impl_listener {
  ($doc: literal, $name: ident, $event_ty: ident) => {
    paste::paste! {
      #[doc= $doc]
      #[derive(Declare, Declare2)]
      pub struct [<$name Listener>]{
        #[declare(skip)]
        [<$name:snake _subject>]: [<$name Subject>]
      }

      impl [<$name ListenerDeclarer>] {
        fn subject(&mut self) -> [<$name Subject>] {
          self
            .[<$name:snake _subject>]
            .get_or_insert_with([<$name Subject>]::default)
            .clone()
        }
      }

      impl [<$name ListenerDeclarer2>] {
        fn subject(&mut self) -> [<$name Subject>] {
          self
            .[<$name:snake _subject>]
            .get_or_insert_with(DeclareInit::default)
            .value()
            .clone()
        }
      }

      impl [<$name Listener>] {
        /// Convert a observable stream of this event.
        pub fn [<$name:snake _stream>](&self) -> [<$name Subject>] {
          self.[<$name:snake _subject>].clone()
        }
      }

      impl EventListener for [<$name Listener>] {
        type Event = $event_ty;
        #[inline]
        fn dispatch(&self, event: &mut Self::Event) {
          self.[<$name:snake _subject>].clone().next(event)
        }
      }

      impl_query_self_only!([<$name Listener>]);
    }
  };
}

#[macro_export]
macro_rules! impl_multi_event_listener {
  (
    $doc: literal, $name: ident,
    $($on_doc: literal, $event_ty: ident),+
  ) => {
    paste::paste! {
      impl_all_event!($name, $($on_doc, $event_ty),+);
      impl_listener!($doc, $name, [<All $name>]);

      impl [<$name ListenerDeclarer2>] {
        $(
          #[doc = "Sets up a function that will be called whenever the `" $event_ty "` is delivered"]
          pub fn [<on_ $event_ty:snake>]<_F>(mut self, handler: impl Into<DeclareInit<_F>>) -> Self
          where
            _F: FnMut(&mut [<$name Event>]) + 'static
          {
            let (v, pipe) = handler.into().unzip();

            let subscribe = move |mut handler: _F| {
              move |e: &mut [<All $name>]| match e {
                [<All $name>]::$event_ty(e) => handler(e),
                _ => {}
              }
            };

            let mut _unsub = self.subject().subscribe(subscribe(v)).unsubscribe_when_dropped();

            if let Some(pipe) = pipe {
              let subject = self.subject().clone();
              pipe.subscribe(move |handler| {
                _unsub = subject.clone().subscribe(subscribe(handler)).unsubscribe_when_dropped();
              });
            }

            self
          }
        )+
      }

      impl [<$name ListenerDeclarer>] {
        $(
          #[doc = "Sets up a function that will be called \
            whenever the `" $event_ty "` is delivered"]
          pub fn [<on_ $event_ty:snake>](
            mut self,
            handler: impl FnMut(&mut [<$name Event>]) + 'static
          ) -> Self {
            self
              .subject()
              .filter_map(
                (|e| match e {
                  [<All $name>]::$event_ty(e) => Some(e),
                  _ => None,
                }) as fn(&mut [<All $name>]) -> Option<&mut [<$name Event>]>
              )
              .subscribe(handler);
            self
          }
        )+
      }
    }
  };
}

#[macro_export]
macro_rules! impl_single_event_listener {
  ($doc: literal, $name: ident) => {
    paste::paste! {
      impl_listener!($doc, $name);

      impl [<$name ListenerDeclarer2>] {
        #[doc = "Sets up a function that will be called whenever the `" [<$name Event>] "` is delivered"]
        pub fn [<on_ $name:snake>](
          self,
          handler: impl FnMut(&'_ mut [<$name Event>]<'_>) + 'static
        ) -> Self {
          self
            .subject()
            .subscribe(handler);
          self
        }
      }

      impl [<$name ListenerDeclarer>] {
        #[doc = "Sets up a function that will be called whenever the `" [<$name Event>] "` is delivered"]
        pub fn [<on_ $name:snake>](
          self,
          handler: impl FnMut(&'_ mut [<$name Event>]<'_>) + 'static
        ) -> Self {
          self
            .subject()
            .subscribe(handler);
          self
        }
      }
    }
  };
}

#[macro_export]
macro_rules! impl_common_event_deref {
  ($event_name: ident) => {
    impl std::ops::Deref for $event_name {
      type Target = CommonEvent;

      #[inline]
      fn deref(&self) -> &Self::Target { &self.common }
    }

    impl std::ops::DerefMut for $event_name {
      #[inline]
      fn deref_mut(&mut self) -> &mut Self::Target { &mut self.common }
    }

    impl std::borrow::Borrow<CommonEvent> for $event_name {
      #[inline]
      fn borrow(&self) -> &CommonEvent { &self.common }
    }

    impl std::borrow::BorrowMut<CommonEvent> for $event_name {
      #[inline]
      fn borrow_mut(&mut self) -> &mut CommonEvent { &mut self.common }
    }
  };
}

#[macro_export]
macro_rules! impl_compose_child_for_listener {
  ($listener: ident) => {
    impl ComposeChild for $listener {
      type Child = Widget;
      #[inline]
      fn compose_child(this: State<Self>, child: Self::Child) -> Widget {
        DataWidget::attach_state(child, this)
      }
    }
  };
}

#[macro_export]
macro_rules! impl_compose_child_with_focus_for_listener {
  ($listener: ident) => {
    impl ComposeChild for $listener {
      type Child = Widget;
      fn compose_child(this: State<Self>, child: Self::Child) -> Widget {
        let child = dynamic_compose_focus_node(child);
        DataWidget::attach_state(child, this)
      }
    }
  };
}
