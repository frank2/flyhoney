use super::*;
use hex;

#[test]
fn test_snes_header() {
    let rom_result = Rom::from_file("test/earthbound.smc");
    assert!(rom_result.is_ok());

    let rom = rom_result.unwrap();
    let snes_header_result = rom.find_valid_snes_header();
    assert!(snes_header_result.is_ok());

    let snes_header = snes_header_result.unwrap();
    panic!("{:?}", snes_header);
}

#[test]
fn test_graphics() {
    let data_1bpp = hex::decode("183c7edbff245a81").unwrap();

    let tile_1bpp_result = SNESTile1BPP::from_data(&data_1bpp);
    assert!(tile_1bpp_result.is_ok());

    let tile_1bpp = tile_1bpp_result.unwrap();
    let expected_1bpp_map: Vec<u8> = [0,0,0,1,1,0,0,0,
                                      0,0,1,1,1,1,0,0,
                                      0,1,1,1,1,1,1,0,
                                      1,1,0,1,1,0,1,1,
                                      1,1,1,1,1,1,1,1,
                                      0,0,1,0,0,1,0,0,
                                      0,1,0,1,1,0,1,0,
                                      1,0,0,0,0,0,0,1].to_vec();

    let colormap_result = tile_1bpp.to_colormap();
    assert!(colormap_result.is_ok());
    
    let colormap = colormap_result.unwrap();
    assert_eq!(colormap, expected_1bpp_map);

    let map_2bpp: Vec<u8> = [2,2,3,3,3,3,1,1,
                             2,2,2,1,1,1,1,1,
                             2,2,3,2,2,1,1,3,
                             2,2,3,1,2,2,2,2,
                             2,2,3,2,2,1,1,1,
                             2,2,3,1,1,1,1,1,
                             3,3,2,0,0,2,2,2,
                             2,2,2,0,0,0,0,0].to_vec();

    let planar_2bpp_result = SNESTile2BPPPlanar::from_colormap(&map_2bpp);
    assert!(planar_2bpp_result.is_ok());

    let planar_2bpp = planar_2bpp_result.unwrap();
    assert_eq!(planar_2bpp.0.to_vec(), hex::decode("3f1f2730273fc000fce0f9eff8e0e7e0").unwrap());

    let intertwined_2bpp_result = SNESTile2BPPIntertwined::from_colormap(&map_2bpp);
    assert!(intertwined_2bpp_result.is_ok());

    let intertwined_2bpp = intertwined_2bpp_result.unwrap();
    assert_eq!(intertwined_2bpp.0.to_vec(), hex::decode("3ffc1fe027f930ef27f83fe0c0e700e0").unwrap());
}
