#[allow(unused_imports)]
use crate::physical_media_journeys::process_contender::*;
use {
    crate::physical_media_journeys::admit_runtime, crate::physical_media_journeys::media_admission,
};

#[test]
fn child_media_contender() {
    use {std::io::Read, std::io::Write};

    let Ok(root) = std::env::var("WORTH_STORE_C4_CHILD_ROOT") else {
        return;
    };
    let expectation = std::env::var("WORTH_STORE_C4_CHILD_EXPECTATION").unwrap();
    let runtime = admit_runtime(std::path::Path::new(&root));
    match runtime
        .try_admit_filesystem_media(media_admission())
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) if expectation == "success" => {
            assert!(matches!(
                media.close(),
                worth_store::physical_runtime::MediaShutdownOutcome::Released(_)
            ));
        }
        worth_proof::TransitionOutcome::Deferred(deferred) if expectation == "deferred" => {
            deferred.into_runtime().abort();
        }
        worth_proof::TransitionOutcome::Success(_media) if expectation == "die" => {
            std::process::exit(0)
        }
        worth_proof::TransitionOutcome::Success(media) if expectation == "hold" => {
            println!("media-lease-ready");
            std::io::stdout().flush().unwrap();
            let mut release = [0_u8; 1];
            std::io::stdin().read_exact(&mut release).unwrap();
            assert!(matches!(
                media.close(),
                worth_store::physical_runtime::MediaShutdownOutcome::Released(_)
            ));
        }
        _ => panic!("child observed the wrong ownership outcome"),
    }
}
