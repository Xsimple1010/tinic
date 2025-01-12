use generics::erro_handle::ErroHandle;
use std::io;
use tinic::{self, args_manager::RetroArgs, DeviceListener, Tinic};

#[derive(Debug, Default)]
struct DeviceEventHandle;

impl DeviceListener for DeviceEventHandle {
    fn connected(&self, device: tinic::Device) {
        println!("connected -> {}", device.name)
    }

    fn disconnected(&self, device: tinic::Device) {
        println!("disconnected -> {}", device.name)
    }

    fn button_pressed(&self, button: String, device: tinic::Device) {
        println!("{} pressed -> {}", device.name, button)
    }
}

#[tokio::main]
async fn main() -> Result<(), ErroHandle> {
    let args = RetroArgs::new()?;

    let event = DeviceEventHandle::default();
    let mut tinic = Tinic::new(Box::new(event))?;

    if let Some(core) = &args.core {
        tinic.load_game(&core, &args.rom).await?;
    } else {
        //baixa as infamações dos cores
        tinic.try_update_core_infos(false).await?;

        let infos = tinic.get_compatibility_info_cores(&args.rom);

        println!("CORES COM POSSÍVEL COMPATIBILIDADE COM ESSA ROM: ");
        for index in 0..infos.len() {
            if let Some(info) = infos.get(index) {
                println!("{} -> {}", index, info.core_name);
            }
        }

        println!("digite o numero do core desejado: ");
        let mut command = String::new();

        match io::stdin().read_line(&mut command) {
            Ok(_) => {
                // let _state = tinic.load_game(&args.core, &args.rom)?;
            }
            Err(_) => {}
        }
    }

    'running: loop {
        println!("Para interagir digite o numero de um dos comandos disponíveis!");
        println!("0: sair");
        println!("1: save state");
        println!("2: load state");
        println!("3: pause");
        println!("4: resume");
        println!("5: reset");
        println!("6: stop rom");

        let mut command = String::new();

        match io::stdin().read_line(&mut command) {
            Ok(..) => {
                if command.starts_with("0") {
                    tinic.quit().await;
                    break 'running;
                } else if command.starts_with("1") {
                    let f = tinic.save_state(1).await;
                    println!("{:?}", f);
                } else if command.starts_with("2") {
                    let d = tinic.load_state(1).await;
                    println!("LOADED -> {:?}", d);
                } else if command.starts_with("3") {
                    tinic.pause();
                } else if command.starts_with("4") {
                    tinic.resume();
                } else if command.starts_with("5") {
                    tinic.reset();
                } else if command.starts_with("6") {
                    tinic.quit().await;
                } else if command.starts_with("7") {
                }

                println!();
            }
            Err(..) => println!("erro ao ler o comando!"),
        }
    }

    Ok(())
}
