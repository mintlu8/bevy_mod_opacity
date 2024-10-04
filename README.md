# bevy_mod_opacity

Hierarchical opacity for bevy.

## The `Opacity` component

When `Opacity` is put on an entity, all its descendants will be affected by the opacity value.
Entity with no `Opacity` ancestor is not affected by this crate.

## Support for native types

We innately support `2d`, `3d` and `ui`, this includes `Sprite`, `Text`, `Handle<StandardMaterial>`,
`Handle<ColorMaterial>`, `Image`, `BackgroundColor` and `ForegroundColor`.

Additionally you can implement `OpacityComponent` or derive `Opacity` to make your own types
ang materials work with this crate, more complicated behaviors can be achieved through `OpacityQuery` by
deriving `QueryData` and implementing `OpacityQuery` manually. This is the preferred way
to add support for third party types.

## `FadeIn` and `FadeOut`

These components adds a quick way to add and remove items from your scenes smoothly
as a complement to `insert` and an alternative to `despawn_recursive`.

## Versions

| bevy | bevy_mod_opacity   |
|------|--------------------|
| 0.14 | 0.1                |

## License

License under either of

Apache License, Version 2.0 (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)
at your option.

## Contribution

Contributions are welcome!

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
