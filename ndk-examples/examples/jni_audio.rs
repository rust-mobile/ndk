use jni::objects::JObject;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
fn main() {
    enumerate_audio_devices().unwrap();
}

const GET_DEVICES_OUTPUTS: jni::sys::jint = 2;

fn enumerate_audio_devices() -> Result<(), Box<dyn std::error::Error>> {
    // Create a VM for executing Java calls
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let context = unsafe { JObject::from_raw(ctx.context().cast()) };
    let env = vm.attach_current_thread()?;

    // Query the global Audio Service
    let class_ctxt = env.find_class("android/content/Context")?;
    let audio_service = env.get_static_field(class_ctxt, "AUDIO_SERVICE", "Ljava/lang/String;")?;

    let audio_manager = env
        .call_method(
            context,
            "getSystemService",
            // JNI type signature needs to be derived from the Java API
            // (ArgTys)ResultTy
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[audio_service],
        )?
        .l()?;

    // Enumerate output devices
    let devices = env.call_method(
        audio_manager,
        "getDevices",
        "(I)[Landroid/media/AudioDeviceInfo;",
        &[GET_DEVICES_OUTPUTS.into()],
    )?;

    println!("-- Output Audio Devices --");

    let device_array = devices.l()?.into_raw();
    let len = env.get_array_length(device_array)?;
    for i in 0..len {
        let device = env.get_object_array_element(device_array, i)?;

        // Collect device information
        // See https://developer.android.com/reference/android/media/AudioDeviceInfo
        let product_name: String = {
            let name =
                env.call_method(device, "getProductName", "()Ljava/lang/CharSequence;", &[])?;
            let name = env.call_method(name.l()?, "toString", "()Ljava/lang/String;", &[])?;
            env.get_string(name.l()?.into())?.into()
        };
        let id = env.call_method(device, "getId", "()I", &[])?.i()?;
        let ty = env.call_method(device, "getType", "()I", &[])?.i()?;

        let sample_rates = {
            let sample_array = env
                .call_method(device, "getSampleRates", "()[I", &[])?
                .l()?
                .into_raw();
            let len = env.get_array_length(sample_array)?;

            let mut sample_rates = vec![0; len as usize];
            env.get_int_array_region(sample_array, 0, &mut sample_rates)?;
            sample_rates
        };

        println!("Device {}: Id {}, Type {}", product_name, id, ty);
        println!("sample rates: {:#?}", sample_rates);
    }

    Ok(())
}
