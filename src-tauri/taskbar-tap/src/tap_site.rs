use std::ffi::c_void;

use windows::Win32::Foundation::{CLASS_E_NOAGGREGATION, E_NOTIMPL};
use windows::Win32::System::Com::{
    COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize, IClassFactory, IClassFactory_Impl,
};
use windows::Win32::System::Ole::{IObjectWithSite, IObjectWithSite_Impl};
use windows_core::{BOOL, GUID, IInspectable, IUnknown, Interface, PCWSTR, Ref, Result, implement};

use crate::INIT_TRANSPARENT;
use crate::bindings::{
    IUIElement, IVisualTreeService, IVisualTreeServiceCallback, IVisualTreeServiceCallback2,
    IVisualTreeServiceCallback2_Impl, IVisualTreeServiceCallback_Impl, IXamlDiagnostics,
    ParentChildRelation, VisualElement, VisualElementState, VisualMutationType,
};
use crate::taskbar_background;

const MUTATION_ADD: VisualMutationType = VisualMutationType(0);
const MUTATION_REMOVE: VisualMutationType = VisualMutationType(1);

#[implement(IClassFactory)]
pub struct TapClassFactory;

impl IClassFactory_Impl for TapClassFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Ref<IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut c_void,
    ) -> Result<()> {
        if !punkouter.is_null() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }

        let site: IObjectWithSite = TapSite.into();
        unsafe { site.query(riid, ppvobject).ok() }
    }

    fn LockServer(&self, _flock: BOOL) -> Result<()> {
        Ok(())
    }
}

#[implement(IObjectWithSite)]
struct TapSite;

impl IObjectWithSite_Impl for TapSite_Impl {
    fn SetSite(&self, punksite: Ref<IUnknown>) -> Result<()> {
        if let Some(site) = punksite.as_ref() {
            let diagnostics: IXamlDiagnostics = site.cast()?;
            let starts_transparent = unsafe { diagnostics.GetInitializationData() }
                .is_ok_and(|data| data == INIT_TRANSPARENT);

            taskbar_background::set_transparent(starts_transparent);
            start_watching(diagnostics);
        }

        Ok(())
    }

    fn GetSite(&self, _riid: *const GUID, _ppvsite: *mut *mut c_void) -> Result<()> {
        Err(E_NOTIMPL.into())
    }
}

struct SendableDiagnostics(IXamlDiagnostics);

// SAFETY: the XAML diagnostics site is free-threaded.
unsafe impl Send for SendableDiagnostics {}

impl SendableDiagnostics {
    fn into_inner(self) -> IXamlDiagnostics {
        self.0
    }
}

// Advising from the thread that delivered the site hangs Explorer, so it runs on a worker.
fn start_watching(diagnostics: IXamlDiagnostics) {
    let diagnostics = SendableDiagnostics(diagnostics);
    let _ = std::thread::Builder::new()
        .name("easy-paper-tap-advise".into())
        .spawn(move || {
            let _ = advise(diagnostics.into_inner());
        });
}

fn advise(diagnostics: IXamlDiagnostics) -> Result<()> {
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).ok()? };

    let result = (|| {
        let tree_service: IVisualTreeService = diagnostics.cast()?;
        let watcher: IVisualTreeServiceCallback2 = VisualTreeWatcher { diagnostics }.into();
        unsafe { tree_service.AdviseVisualTreeChange(&watcher.cast::<IVisualTreeServiceCallback>()?) }
    })();

    unsafe { CoUninitialize() };
    result
}

// XAML only delivers tree changes to watchers that implement the v2 callback.
#[implement(IVisualTreeServiceCallback2)]
struct VisualTreeWatcher {
    diagnostics: IXamlDiagnostics,
}

impl VisualTreeWatcher {
    fn resolve(&self, handle: u64) -> Result<IUIElement> {
        let inspectable: IInspectable = unsafe { self.diagnostics.GetIInspectableFromHandle(handle) }?;
        inspectable.cast()
    }
}

impl IVisualTreeServiceCallback_Impl for VisualTreeWatcher_Impl {
    fn OnVisualTreeChange(
        &self,
        relation: &ParentChildRelation,
        element: &VisualElement,
        mutationtype: VisualMutationType,
    ) -> Result<()> {
        match mutationtype {
            MUTATION_ADD => {
                let is_background = taskbar_background::track_added(
                    relation.Parent,
                    element.Handle,
                    &element.Type.to_string(),
                    &element.Name.to_string(),
                );
                if is_background && let Ok(ui_element) = self.resolve(element.Handle) {
                    taskbar_background::register(element.Handle, ui_element);
                }
            }
            MUTATION_REMOVE => taskbar_background::track_removed(element.Handle),
            _ => {}
        }

        Ok(())
    }
}

impl IVisualTreeServiceCallback2_Impl for VisualTreeWatcher_Impl {
    fn OnElementStateChanged(
        &self,
        _element: u64,
        _elementstate: VisualElementState,
        _context: &PCWSTR,
    ) -> Result<()> {
        Ok(())
    }
}
