use std::{path::Path, ptr};
use windows::{
    core::Interface,
    Win32::{
        Foundation::BSTR,
        System::{
            Com::{CoCreateInstance, CoInitialize, CoUninitialize, CLSCTX_INPROC_SERVER},
            TaskScheduler::{
                IExecAction, ITaskService, TaskScheduler, TASK_ACTION_EXEC, TASK_ACTION_TYPE,
                TASK_CREATE_OR_UPDATE, TASK_LOGON_NONE, TASK_LOGON_TYPE, TASK_RUNLEVEL_HIGHEST,
                TASK_RUNLEVEL_LUA, TASK_RUNLEVEL_TYPE, TASK_RUN_IGNORE_CONSTRAINTS,
                TASK_TRIGGER_BOOT, TASK_TRIGGER_TYPE2,
            },
        },
    },
};

/// Runs a task on the Windows Task Scheduler.
pub fn run_task(task: impl AsRef<Path>) -> crate::Result<()> {
    let task = task.as_ref();

    let task_folder = task.parent().unwrap().to_str().unwrap();
    let task_name = task.file_name().unwrap().to_str().unwrap();

    println!("[+] Running the {} task...", task.display());

    unsafe { CoInitialize(ptr::null_mut())? };

    let task_service = unsafe {
        let task_service =
            CoCreateInstance::<_, ITaskService>(&TaskScheduler, None, CLSCTX_INPROC_SERVER)?;
        task_service.Connect(None, None, None, None)?;
        task_service
    };

    unsafe {
        let task = task_service.GetFolder(task_folder)?.GetTask(task_name)?;
        task.RunEx(None, TASK_RUN_IGNORE_CONSTRAINTS.0, 0, None)?;
    }

    unsafe { CoUninitialize() };

    Ok(())
}

/// Creates a scheduled task on the Windows Task Scheduler. If the task already exists, it will be deleted and recreated.
pub fn create_task(
    path: impl AsRef<Path>,
    command: impl Into<String>,
    highest_privilege: bool,
) -> crate::Result<()> {
    let path = path.as_ref();

    let task_folder = path.parent().unwrap().to_str().unwrap();
    let task_name = path.file_name().unwrap().to_str().unwrap();

    println!("[+] Creating the {} task...", path.display());

    unsafe { CoInitialize(ptr::null_mut())? };

    let task_service =
        unsafe { CoCreateInstance::<_, ITaskService>(&TaskScheduler, None, CLSCTX_INPROC_SERVER)? };

    unsafe {
        task_service.Connect(None, None, None, None)?;
    }

    let task_folder = unsafe { task_service.GetFolder(task_folder)? };
    let task = unsafe { task_service.NewTask(0)? };

    unsafe {
        let task_triggers = task.Triggers()?;

        // Set the task to run on startup.
        task_triggers.Create(TASK_TRIGGER_TYPE2(TASK_TRIGGER_BOOT.0))?;

        task.SetTriggers(task_triggers)?;
    }

    let mut execution_time_limit = BSTR::from("Nothing");
    unsafe {
        let task_settings = task.Settings()?;

        task_settings.ExecutionTimeLimit(&mut execution_time_limit)?;

        task.SetSettings(task_settings)?;
    }

    unsafe {
        let task_principal = task.Principal()?;

        task_principal.SetRunLevel(TASK_RUNLEVEL_TYPE(if highest_privilege {
            TASK_RUNLEVEL_HIGHEST.0
        } else {
            TASK_RUNLEVEL_LUA.0
        }))?;

        task_principal.SetUserId("SYSTEM")?;

        task.SetPrincipal(task_principal)?;
    }

    unsafe {
        let task_actions = task.Actions()?;

        let action = task_actions
            // Create an execute action.
            .Create(TASK_ACTION_TYPE(TASK_ACTION_EXEC.0))?
            .cast::<IExecAction>()?;

        action.SetPath(command.into())?;

        task.SetActions(task_actions)?;
    }

    unsafe {
        task_folder.RegisterTaskDefinition(
            task_name,
            task,
            TASK_CREATE_OR_UPDATE.0,
            None,
            None,
            TASK_LOGON_TYPE(TASK_LOGON_NONE.0),
            None,
        )?;
    }

    Ok(())
}
