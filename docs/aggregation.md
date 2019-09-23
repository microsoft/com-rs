# Aggregation

COM allows you to aggregate other COM objects. This means exposing their interfaces as your own, allowing code reuse.

If you plan to use aggregation, then we assume you are somewhat familiar with the inner workings of COM. This explanation assumes the same.

We will walk you through producing a `WindowsFileManager`, which aggregates another COM object, the `LocalFileManager`. Specifically, we choose to aggregate the `ILocalFileManager` interface from `LocalFileManager`.

1. Define an aggregable coclass. Here we use the `co_class` macro but with the `aggregatable` attribute like so: `#[co_class(..., aggregatable)]`.

```rust
use com::co_class;

#[co_class(implements(ILocalFileManager), aggregatable)]
pub struct LocalFileManager {
    user_field_one: u32,
    user_field_two: u32,
}

impl ILocalFileManager for LocalFileManager {
    fn delete_local(&self) -> HRESULT {
        println!("Deleting Locally...");
        NOERROR
    }
}

impl LocalFileManager {
    pub(crate) fn new() -> Box<LocalFileManager> {
        let user_field_one = 20;
        let user_field_two = 40;
        LocalFileManager::allocate(user_field_one, user_field_two)
    }
}
```

2. Define the class that will aggregate `LocalFileManager`. This can be aggregable or not.
- You are responsible for instantiating your aggregates.
- In order for us to generate the correct QueryInterface implementation, you need to tell us which interfaces **EACH** aggregated object is exposing. To do this, you use a `aggregates(...)` attribute argument for **EACH** aggregated object. The order in which these interfaces are defined doesn't matter.

```rust
#[co_class(implements(IFileManager), aggregates(ILocalFileManager))]
pub struct WindowsFileManager {
    user_field_one: u32,
    user_field_two: u32,
}


impl IFileManager for WindowsFileManager {
    fn delete_all(&self) -> HRESULT {
        println!("Deleting all by delegating to Local and Remote File Managers...");
        NOERROR
    }
}
```

3. Define the class constructor.

Here, we chose to instantiate the aggregate in the constructor. Each aggregated object is initialised as NULL, until the aggregate is instantiated, through the `set_aggregate_*` methods. Hence, you could choose to instantiate aggregates whenever you want.

In order to instantiate the aggregate, you will need to
- Create the aggregated object as an aggregate. This can be done through CoCreateInstance,
- Supply the resultant IUnknown interface pointer to the appropriate `set_aggregate_*` methods. For each base interface exposed, there will be a separate `set_aggregate_*` method defined. Setting aggregate for one base interface will set it for every base interface exposed by the same aggregated object.

In this case, we are exposing only the `ILocalFileManager` as aggregated interfaces. This means a `set_aggregate_ilocal_file_manager` will be generated, which we can use to instantiate the underlying aggregated object.

```rust
impl WindowsFileManager {
    pub(crate) fn new() -> Box<WindowsFileManager> {
        //Initialise the COM object.
        let user_field_one = 20;
        let user_field_two = 40;
        let mut wfm = WindowsFileManager::allocate(user_field_one, user_field_two);

        let runtime = Runtime::new().expect("Failed to get runtime!");
        let iunknown = runtime
            .create_aggregated_instance::<dyn IUnknown, WindowsFileManager>(
                &CLSID_LOCAL_FILE_MANAGER_CLASS,
                &mut *wfm,
            )
            .expect("Failed to instantiate aggregate!");

        wfm.set_aggregate_ilocal_file_manager(iunknown);

        wfm
    }
}

```