# GPC-Proyecto2

Un **renderizador de ray tracing** en 3D hecho en **Rust** usando la biblioteca `raylib`. Incluye efectos como **ciclo de día/noche**, **sol visible**, **rotación de escena**, y renderizado multihilo.

## Características

- **Ray Tracing en tiempo real** con sombras, reflexiones y texturas.
- **Ciclo de día/noche** con sol que se mueve.
- **Cámara orbital** con teclas de flecha.
- **Render multihilo** con `rayon` para mayor velocidad.
- **Texturas** para bloques (madera, césped, agua, hojas rosadas).
- **Minecraft-like diorama** generado proceduralmente.

## Requisitos

- [Rust](https://www.rust-lang.org/tools/install) (versión 1.70 o superior)
- [Cargo](https://doc.rust-lang.org/cargo/)
- Bibliotecas nativas para `raylib` (instaladas automáticamente por `raylib-sys`)

## Cómo correr

1. Clona el repositorio:

```bash
git clone <https://github.com/AleWWH1104/GPC-Proyecto2.git>
cd <GPC-Proyecto2>
```

2. Asegúrate de tener las imágenes en la carpeta `assets/`:

```
assets/
├── wood.png
├── grass.png
├── water.png
└── pink_leaves.png
```

3. Ejecuta el proyecto:

```bash
cargo run
```

Puedes compilar en modo `release` para mayor velocidad:

```bash
cargo run --release
```

## Dependencias
- `raylib`
- `rayon`

## Video diorama
https://youtu.be/8YNlFCG3EXY