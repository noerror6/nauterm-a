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
import 'window/native_windowing.dart' as windowing;
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
  final register = library
      .lookupFunction<
        Bool Function(Pointer<Utf8>, Pointer<Uint8>, IntPtr),
        bool Function(Pointer<Utf8>, Pointer<Uint8>, int)
      >('nauterm_android_register_credential_key');
  for (final account in ['local-database-key', 'device-key']) {
    final platformBytes = await _credentialChannel.invokeMethod<Uint8List>(
      'getOrCreateKey',
      {'account': account},
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
    implements windowing.NautermRootController {
  final NautermWorkspaceController _workspaceController =
      NautermWorkspaceController();
  final GlobalKey<NavigatorState> _navigatorKey = GlobalKey<NavigatorState>();
  bool _settingsOpen = false;

  @override
  void initState() {
    super.initState();
    windowing.registerNautermRoot(this);
  }

  @override
  void dispose() {
    windowing.unregisterNautermRoot(this);
    _workspaceController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => NautermApp(
    navigatorKey: _navigatorKey,
    workspaceController: _workspaceController,
    onOpenSettings: showSettingsWindow,
    onOpenTerminalSettings: () => windowing.showSettingsWindow(
      page: windowing.NautermSettingsPage.terminal,
    ),
  );

  @override
  void showSettingsWindow() {
    if (_settingsOpen) return;
    final navigator = _navigatorKey.currentState;
    if (navigator == null) return;
    _settingsOpen = true;
    navigator
        .push(
          MaterialPageRoute<void>(
            builder: (_) =>
                const Scaffold(body: SafeArea(child: SettingsPanel())),
          ),
        )
        .whenComplete(() => _settingsOpen = false);
  }

  @override
  void hideSettingsWindow() {
    if (_settingsOpen) _navigatorKey.currentState?.pop();
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
