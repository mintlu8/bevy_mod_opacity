# bevy_mod_opacity

Hierarchical opacity for bevy.

## The `Opacity` component

When `Opacity` is inserted to an entity, the entity and all its descendants
will be affected by the opacity value. Unlike bevy components like `Visibility`
`Opacity` does not need to be put on every entity in the tree.
Entities with no `Opacity` ancestor will not be affected by this crate.

## Support for native types

We innately support `2d`, `3d` and `ui`, this includes `Sprite`, `TextColor`, `StandardMaterial`,
`ColorMaterial`, `Image`, `BackgroundColor` and `ForegroundColor`.

Additionally you can implement `OpacityQuery` or derive `Opacity` to make your own types
and materials work with this crate. Combining `OpacityQuery` with custom `QueryData` can
add support for third party types.

## `FadeIn` and `FadeOut`

These components adds a quick way to add and remove entities from your scenes smoothly.
You should add a `FadeIn` during the `spawn` call and use `entity.insert(FadeOut)` instead
of `entity.despawn_recursive()`

## FAQ

* My 3d scene is not fading correctly

 Ensure materials are duplicated and unique, since we write to the underlying material directly.
 Also make sure `AlphaMode` is set to `Blend` if applicable.

## Versions

| bevy | bevy_mod_opacity   |
|------|--------------------|
| 0.14 | 0.1                |
| 0.15 | 0.2                |

## License

License under either of

Apache License, Version 2.0 (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)
at your option.

## Contribution

Contributions are welcome!

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
