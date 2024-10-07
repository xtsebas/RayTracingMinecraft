# Diorama 3D en Rust

Este proyecto es una representación de un diorama 3D creado en Rust que incluye múltiples cubos (bloques) que forman columnas, paredes y otros elementos. El proyecto utiliza múltiples materiales como `OLDWOOD`, `STONE`, `RUBBER`, `GLASS`, entre otros, para darle textura y profundidad al escenario. Además, cuenta con funcionalidades como trazado de rayos y efectos de iluminación.

## Requisitos del sistema

Para ejecutar este proyecto, necesitas tener instalado:

- [Rust](https://www.rust-lang.org/): Asegúrate de tener la última versión estable de Rust.
- [Cargo](https://doc.rust-lang.org/cargo/): Viene por defecto con la instalación de Rust.

Adicionalmente, este proyecto hace uso de varias bibliotecas de terceros:

- `nalgebra_glm` para operaciones matemáticas en 3D.
- `minifb` para crear ventanas y renderizar el framebuffer.
- `once_cell` para la inicialización de variables estáticas.
- Otras bibliotecas personalizadas incluidas en el código fuente.

## Instalación

Sigue los siguientes pasos para clonar e instalar el proyecto en tu máquina local:

1. Clona el repositorio:
    ```bash
    git clone https://github.com/tu-usuario/tu-repositorio.git
    cd tu-repositorio
    ```

2. Asegúrate de tener instaladas las dependencias de Rust. Si no las tienes, instálalas con `cargo`:
    ```bash
    cargo build
    ```

## Ejecución del proyecto

Después de haber realizado la instalación, puedes ejecutar el proyecto usando `cargo`:

```bash
cargo run
```

## VIDEO DE PRUEBA

https://github.com/user-attachments/assets/8c798a59-6864-418d-b770-3a174a8e9c3d

