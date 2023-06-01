# New RustSeeker GUI

A new RustSeeker GUI, taking adventage of the libadwaita widgets. This new UI is much more beatiful than the old one.

Also, you can change from the dark theme and the light theme with the YAML settings file, besides, with this configuration file you can configure how the log system will show his messages!

#### Settiings documentation

Theres one value that need to be explained:

```yaml
logs_configurations:
  write_to_stdout: false
  write_to_file: true
interface_configurations:
  enable_adw: true
  color_scheme: dark
general:
  skip_metadata_errors: false
```

In the `color_scheme` field you need to choose from dark or light. And that's it XD.