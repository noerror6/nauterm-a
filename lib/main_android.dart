import 'dart:io';
import 'dart:ffi';

import 'package:ffi/ffi.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'app/nauterm_app.dart';
import 'app/nauterm_log.dart';
import 'app/third_party_licenses.dart';
import 'data/nauterm_config_store.dart';
import 'data/nauterm_paths.dart';
import 'settings/settings_panel.dart';
import 'terminal/terminal_config.dart';
import 'terminal/terminal_ffi.dart';
import 'terminal/terminal_models.dart';
import 'window/native_windowing.dart';
import 'workspace/nauterm_workspace.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await NautermPaths.initializeAndroid();
  final paths = NautermPaths.resolve();
  NautermLog.initialize(paths.runtimeLogsDirectory);
  await _registerAndroidCredentials();
  registerNautermThirdPartyLicenses();
  initializeNativeTerminalRuntime();
  try {
    applyNautermRuntimeSettings(
      await NautermConfigStore(paths).loadRuntimeSettings(),
    );
  } on Object catch (error, stackTrace) {
    NautermLog.warning(
      'configuration',
      'Unable to load runtime settings; using defaults.',
      error: error,
      stackTrace: stackTrace,
    );
  }
  // Android native builds currently use Alacritty; Ghostty is not bundled.
  terminalEmulatorBackend = TerminalEmulatorBackend.alacritty;
  assert(Platform.isAndroid);
  runApp(const _AndroidNautermRoot());
}

const _credentialChannel = MethodChannel('com.korvect.nauterm/credential_keys');

/// Install both encrypted keys before any native database or terminal opens.
Future<void> _registerAndroidCredentials() async {
  final library = DynamicLibrary.open('libnauterm_ffi.so');
  final register = library.lookupFunction<
    Bool Function(Pointer<Utf8>, Pointer<Uint8>, IntPtr),
    bool Function(Pointer<Utf8>, Pointer<Uint8>, int)
  >('nauterm_android_register_credential_key');
  for (final account in ['local-database-key', 'device-key']) {
    final platformBytes = await _credentialChannel.invokeMethod<Uint8List>(
      'getOrCreateKey', {'account': account},
    );
    if (platformBytes == null || platformBytes.length != 32) {
      throw StateError('Android secure credential storage is unavailable');
    }
    // Platform channel buffers may be unmodifiable; only wipe our own copy.
    final bytes = Uint8List.fromList(platformBytes);
    final name = account.toNativeUtf8();
    final key = calloc<Uint8>(bytes.length);
    try {
      key.asTypedList(bytes.length).setAll(0, bytes);
      if (!register(name, key, bytes.length)) {
        throw StateError('Unable to register Android credential key');
      }
    } finally {
      key.asTypedList(bytes.length).fillRange(0, bytes.length, 0);
      bytes.fillRange(0, bytes.length, 0);
      calloc.free(key);
      calloc.free(name);
    }
  }
}

class _AndroidNautermRoot extends StatefulWidget {
  const _AndroidNautermRoot();

  @override
  State<_AndroidNautermRoot> createState() => _AndroidNautermRootState();
}

class _AndroidNautermRootState extends State<_AndroidNautermRoot>
    implements NautermRootController {
  final NautermWorkspaceController _workspaceController =
      NautermWorkspaceController();
  bool _settingsOpen = false;

  @override
  void initState() {
    super.initState();
    registerNautermRoot(this);
  }

  @override
  void dispose() {
    unregisterNautermRoot(this);
    _workspaceController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => PopScope(
    canPop: !_settingsOpen,
    onPopInvokedWithResult: (didPop, _) {
      if (!didPop && _settingsOpen) hideSettingsWindow();
    },
    child: NautermApp(
      workspaceController: _workspaceController,
      onOpenSettings: showSettingsWindow,
      onOpenTerminalSettings: showSettingsWindow,
      settingsOverlay: _settingsOpen
          ? const Material(child: SafeArea(child: SettingsPanel()))
          : null,
    ),
  );

  @override
  void showSettingsWindow() {
    if (mounted) setState(() => _settingsOpen = true);
  }

  @override
  void hideSettingsWindow() {
    if (mounted) setState(() => _settingsOpen = false);
  }

  @override
  void showMainWindow() => hideSettingsWindow();

  @override
  Future<void> closeMainWindow() => requestQuit().then((_) {});

  @override
  void closeSelectedTerminalTab() =>
      _workspaceController.closeSelectedTerminalTab();

  @override
  Future<bool> requestQuit() async {
    if (!await _workspaceController.confirmQuitIfNeeded()) return false;
    await _workspaceController.flushAndClose();
    return true;
  }
}
