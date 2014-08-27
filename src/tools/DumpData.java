import java.util.Map;

// Updated for 1.8-pre2.
// Follow the grep commands to update obfuscated class names.

public class DumpData {
    public static void main(String[] args) {
// grep -C1 'static boolean . = false;' * | grep 'static final Logger . = LogManager\.getLogger();' # od
        od.c();
// grep '"textures/atlas/blocks.png"' * # cty
// grep 'private final cty b;' * # cxi
        cxi manager = new cxi(new cty("textures"));
        Map block_to_blockstate = manager.c().a().a();

        System.out.print("// (name, temperature, humidity)\n");
        System.out.print("pub static BIOMES: [Option<(&'static str, f32, f32)>, ..256] = [");
        int none_trail_len = 0;
// grep Swampland * # ark
        for(ark biome : ark.n()) {
            if(biome == null) {
                if((none_trail_len % 16) == 0)
                    System.out.print("\n    None,");
                else
                    System.out.print(" None,");
                none_trail_len++;
            } else {
                System.out.printf("\n    Some((\"%s\", %.1f, %.1f)),", biome.ah, biome.ap, biome.aq);
                none_trail_len = 0;
            }
        }
        System.out.print("\n];\n\n");

        System.out.print("// (id, name, variant)\n");
        System.out.print("pub static BLOCK_STATES: &'static [(u16, &'static str, &'static str)] = &[\n");
// grep 'new \w*("air");' * # atp
        for(Object o : atp.d) {
            int id = atp.d.b(o);
// grep '?"normal":' * # cxj
            cxj path = (cxj)block_to_blockstate.get(o);
            if(path == null || !path.b().equals("minecraft"))
                System.out.printf("    // %04x: \"%s\" (%s)\n", id, o, path);
            else
                System.out.printf("    (0x%04x, \"%s\", \"%s\"),\n", id, path.a(), path.c());
        }
        System.out.print("];\n");
    }
}
