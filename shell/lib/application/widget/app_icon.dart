import 'package:flutter/material.dart';
import 'package:freedesktop_desktop_entry/freedesktop_desktop_entry.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:material_design_icons_flutter/material_design_icons_flutter.dart';
import 'package:shell/application/provider/image_from_icon_query.dart';
import 'package:shell/application/provider/localized_desktop_entries.dart';
import 'package:shell/wayland/model/wl_surface.dart';
import 'package:shell/wayland/provider/xdg_toplevel_state.dart';

class AppIconByPath extends StatelessWidget {
  const AppIconByPath({
    required this.path,
    super.key,
  });

  final String? path;

  @override
  Widget build(BuildContext context) {
    return Consumer(
      builder: (_, WidgetRef ref, __) {
        if (path == null) {
          return const SizedBox();
        }
        return _buildIcon(ref, path!);
      },
    );
  }

  Widget _buildIcon(WidgetRef ref, String path) {
    return LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        final asyncValue = ref.watch(
          imageFromIconQueryProvider(
            IconQuery(
              name: path,
              size: constraints.biggest.shortestSide.floor(),
              extensions: const ['svg', 'png'],
            ),
            constraints.biggest,
          ),
        );
        if (!asyncValue.hasValue) {
          return const SizedBox();
        }
        final rawImage = asyncValue.value;
        return RawImage(
          image: rawImage,
          fit: BoxFit.contain,
        );
      },
    );
  }
}

class AppIconById extends ConsumerWidget {
  const AppIconById({
    required this.id,
    super.key,
  });

  final String? id;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    if (id == null) {
      return Icon(MdiIcons.helpCircle);
    }

    return ref.watch(localizedDesktopEntryForIdProvider(id!)).maybeWhen(
          data: (entry) {
            if (entry == null) {
              return Icon(MdiIcons.helpCircle);
            }
            final iconPath = entry.entries[DesktopEntryKey.icon.string];
            if (iconPath == null) {
              return Icon(MdiIcons.helpCircle);
            }
            return AppIconByPath(path: iconPath);
          },
          orElse: () => Icon(MdiIcons.helpCircle),
        );
  }
}

class AppIconByViewId extends ConsumerWidget {
  const AppIconByViewId({
    required this.surfaceId,
    super.key,
  });

  final SurfaceId surfaceId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final appId =
        ref.watch(xdgToplevelStateProvider(surfaceId).select((v) => v.appId));
    return AppIconById(id: appId);
  }
}
